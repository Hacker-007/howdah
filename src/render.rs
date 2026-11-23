//! Writing SQL. Scope is checked while writing.

use crate::error::Error;
use crate::expr::Node;
use crate::select::Select;
use crate::table::SourceId;

struct Writer {
    sql: String,
    /// The one table a column may come from.
    scope: SourceId,
}

impl Writer {
    fn new(scope: SourceId) -> Self {
        Self {
            sql: "SELECT".to_string(),
            scope,
        }
    }

    fn node(&mut self, node: &Node) -> Result<(), Error> {
        match node {
            Node::Column(column) => {
                if column.source != self.scope {
                    return Err(Error::NotInScope {
                        table: column.table,
                        column: column.name,
                    });
                }

                self.ident(column.table);
                self.sql.push('.');
                self.ident(column.name);
            }
        }

        Ok(())
    }

    fn ident(&mut self, name: &str) {
        self.sql.push('"');
        self.sql.push_str(&name.replace('"', "\"\""));
        self.sql.push('"');
    }
}

pub(crate) fn select<R>(select: &Select<R>) -> Result<String, Error> {
    let mut w = Writer::new(select.source);
    for (i, column) in select.columns.iter().enumerate() {
        w.sql.push_str(if i == 0 { " " } else { ", " });
        w.node(column)?;
    }

    w.sql.push_str(" FROM ");
    w.ident(select.from);
    Ok(w.sql)
}
