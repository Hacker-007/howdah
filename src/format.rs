//! Writing SQL. Scope is checked while writing.

use crate::error::Error;
use crate::expression::{CompareOperation, Direction, ErasedExpression, LogicOperation};
use crate::select::{JoinKind, Select};
use crate::table::SourceId;
use crate::value::Value;

/// How tightly an expression binds, loosest first, following Postgres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Or,
    And,
    Not,
    Is,
    Comparison,
    /// A column or parameter, which never needs parentheses.
    Atom,
}

impl Precedence {
    fn of(expression: &ErasedExpression) -> Self {
        match expression {
            ErasedExpression::Logic {
                op: LogicOperation::Or,
                ..
            } => Self::Or,
            ErasedExpression::Logic {
                op: LogicOperation::And,
                ..
            } => Self::And,
            ErasedExpression::Not(_) => Self::Not,
            ErasedExpression::IsNull(_) | ErasedExpression::IsNotNull(_) => Self::Is,
            ErasedExpression::Compare { .. } => Self::Comparison,
            ErasedExpression::Column(_) | ErasedExpression::Value(_) => Self::Atom,
        }
    }
}

struct SqlFormatter {
    /// The alias of each use of a table, in FROM order.
    aliases: Vec<(SourceId, String)>,
    /// How many of the aliases a column may come from: all of them outside
    /// ON, and the FROM items up to and including the join whose ON is being
    /// written.
    visible: usize,
    /// The values of `$1`, `$2`, ... in order.
    parameters: Vec<Value>,
}

impl SqlFormatter {
    fn expression(&mut self, expression: &ErasedExpression) -> Result<String, Error> {
        let fragment = match expression {
            ErasedExpression::Column(column) => {
                let alias = self
                    .aliases
                    .iter()
                    .take(self.visible)
                    .find(|(source, _)| *source == column.source)
                    .map(|(_, alias)| alias);
                let Some(alias) = alias else {
                    return Err(Error::NotInScope {
                        table: column.table,
                        column: column.name,
                    });
                };

                format!(
                    "{}.{}",
                    self.identifier(alias),
                    self.identifier(column.name)
                )
            }
            ErasedExpression::Compare { op, left, right } => {
                let op = match op {
                    CompareOperation::Equal => "=",
                    CompareOperation::NotEqual => "<>",
                    CompareOperation::LessThan => "<",
                    CompareOperation::LessThanEqual => "<=",
                    CompareOperation::GreaterThan => ">",
                    CompareOperation::GreaterThanEqual => ">=",
                };

                format!(
                    "{} {op} {}",
                    self.operand(left, Precedence::Atom)?,
                    self.operand(right, Precedence::Atom)?
                )
            }
            ErasedExpression::Value(value) => {
                self.parameters.push(value.clone());
                format!("${}", self.parameters.len())
            }
            ErasedExpression::Logic { op, left, right } => {
                let (op, precedence) = match op {
                    LogicOperation::And => ("AND", Precedence::And),
                    LogicOperation::Or => ("OR", Precedence::Or),
                };

                format!(
                    "{} {op} {}",
                    self.operand(left, precedence)?,
                    self.operand(right, precedence)?
                )
            }
            ErasedExpression::Not(operand) => {
                format!("NOT {}", self.operand(operand, Precedence::Not)?)
            }
            ErasedExpression::IsNull(operand) => {
                // A comparison binds tighter than IS and could go bare, but
                // `a = b IS NULL` reads as if IS applied to `b` alone.
                format!("{} IS NULL", self.operand(operand, Precedence::Atom)?)
            }
            ErasedExpression::IsNotNull(operand) => {
                format!("{} IS NOT NULL", self.operand(operand, Precedence::Atom)?)
            }
        };

        Ok(fragment)
    }

    fn identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    /// An operand of an operator, in parentheses if it binds more loosely
    /// than `precedence`.
    fn operand(
        &mut self,
        expression: &ErasedExpression,
        precedence: Precedence,
    ) -> Result<String, Error> {
        let fragment = self.expression(expression)?;
        if Precedence::of(expression) < precedence {
            Ok(format!("({fragment})"))
        } else {
            Ok(fragment)
        }
    }

    /// Each expression's SQL, separated by `separator`, in parentheses if it
    /// binds more loosely than `precedence`.
    fn list(
        &mut self,
        expressions: &[ErasedExpression],
        separator: &str,
        precedence: Precedence,
    ) -> Result<String, Error> {
        let parts = expressions
            .iter()
            .map(|expression| self.operand(expression, precedence))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(parts.join(separator))
    }
}

/// A name for each use of a table in the statement: the first `users` is
/// `"users"`, the next `"users_2"`, and so on. An alias already taken is
/// skipped.
fn aliases<R>(select: &Select<R>) -> Vec<(SourceId, String)> {
    let items = std::iter::once((select.source, select.from))
        .chain(select.joins.iter().map(|join| (join.source, join.table)));

    let mut aliases = Vec::new();
    for (source, table) in items {
        let mut alias = table.to_string();
        for n in 2.. {
            if !aliases
                .iter()
                .any(|(_, taken): &(SourceId, String)| *taken == alias)
            {
                break;
            }
            alias = format!("{table}_{n}");
        }
        aliases.push((source, alias));
    }

    aliases
}

pub(crate) fn select<R>(select: &Select<R>) -> Result<(String, Vec<Value>), Error> {
    let mut f = SqlFormatter {
        aliases: aliases(select),
        visible: 1 + select.joins.len(),
        parameters: Vec::new(),
    };
    let mut sql = "SELECT".to_string();
    if !select.columns.is_empty() {
        sql.push(' ');
        f.list(&select.columns, ", ", Precedence::Or)
            .map(|fragment| sql.push_str(&fragment))?;
    }

    sql.push_str(" FROM ");
    sql.push_str(&f.identifier(select.from));

    for (i, join) in select.joins.iter().enumerate() {
        sql.push_str(match join.kind {
            JoinKind::Inner => " JOIN ",
            JoinKind::Left => " LEFT JOIN ",
        });
        sql.push_str(&f.identifier(join.table));
        let alias = f.aliases[i + 1].1.clone();
        if alias != join.table {
            sql.push_str(" AS ");
            sql.push_str(&f.identifier(&alias));
        }

        sql.push_str(" ON ");
        // An ON may use the FROM items up to and including this join.
        f.visible = i + 2;
        let on = f.expression(&join.on)?;
        sql.push_str(&on);
    }

    f.visible = f.aliases.len();

    if !select.filters.is_empty() {
        // Filters are joined with AND, which an OR filter must not leak into.
        let precedence = if select.filters.len() > 1 {
            Precedence::And
        } else {
            Precedence::Or
        };

        sql.push_str(" WHERE ");
        f.list(&select.filters, " AND ", precedence)
            .map(|fragment| sql.push_str(&fragment))?;
    }

    if !select.orders.is_empty() {
        sql.push_str(" ORDER BY ");
        let orders = select
            .orders
            .iter()
            .map(|order| {
                let expression = f.operand(&order.expression, Precedence::Or)?;
                let direction = match order.direction {
                    Direction::Asc => "ASC",
                    Direction::Desc => "DESC",
                };
                Ok(format!("{expression} {direction}"))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        sql.push_str(&orders.join(", "));
    }

    if let Some(limit) = select.limit {
        sql.push_str(&format!(" LIMIT {limit}"));
    }

    if let Some(offset) = select.offset {
        sql.push_str(&format!(" OFFSET {offset}"));
    }

    Ok((sql, f.parameters))
}
