//! Writing SQL. Scope is checked while writing.

use crate::error::Error;
use crate::expression::{CompareOperation, ErasedExpression};
use crate::select::Select;
use crate::table::SourceId;

struct SqlFormatter {
    /// The one table a column may come from.
    scope: SourceId,
}

impl SqlFormatter {
    fn new(scope: SourceId) -> Self {
        Self { scope }
    }

    fn expression(&self, expression: &ErasedExpression) -> Result<String, Error> {
        let fragment = match expression {
            ErasedExpression::Column(column) if column.source != self.scope => {
                return Err(Error::NotInScope {
                    table: column.table,
                    column: column.name,
                });
            }
            ErasedExpression::Column(column) => {
                format!(
                    "{}.{}",
                    self.identifier(column.table),
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

                format!("{} {op} {}", self.operand(left)?, self.operand(right)?)
            }
        };

        Ok(fragment)
    }

    fn identifier(&self, name: &str) -> String {
        format!("\"{}\"", name.replace('"', "\"\""))
    }

    /// An operand of an operator, in parentheses unless it is a column. SQL
    /// does not allow `a = b = c`.
    fn operand(&self, expression: &ErasedExpression) -> Result<String, Error> {
        let fragment = self.expression(expression)?;
        match expression {
            ErasedExpression::Column(_) => Ok(fragment),
            _ => Ok(format!("({fragment})")),
        }
    }

    /// Each expression's SQL, separated by `separator`.
    fn list(&self, expressions: &[ErasedExpression], separator: &str) -> Result<String, Error> {
        let parts = expressions
            .iter()
            .map(|expression| self.expression(expression))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(parts.join(separator))
    }
}

pub(crate) fn select<R>(select: &Select<R>) -> Result<String, Error> {
    let f = SqlFormatter::new(select.source);
    let mut sql = "SELECT".to_string();
    if !select.columns.is_empty() {
        sql.push(' ');
        f.list(&select.columns, ", ")
            .map(|fragment| sql.push_str(&fragment))?;
    }

    sql.push_str(" FROM ");
    sql.push_str(&f.identifier(select.from));
    if !select.filters.is_empty() {
        sql.push_str(" WHERE ");
        f.list(&select.filters, " AND ")
            .map(|fragment| sql.push_str(&fragment))?;
    }

    Ok(sql)
}
