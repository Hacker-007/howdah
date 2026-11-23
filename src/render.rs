//! Writing SQL.

use crate::expr::Node;
use crate::select::Select;

struct Writer {
    sql: String,
}

impl Writer {
    fn node(&mut self, node: &Node) {
        match node {
            Node::Column(column) => {
                self.ident(column.table);
                self.sql.push('.');
                self.ident(column.name);
            }
        }
    }

    fn ident(&mut self, name: &str) {
        self.sql.push('"');
        self.sql.push_str(&name.replace('"', "\"\""));
        self.sql.push('"');
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self {
            sql: "SELECT".to_string(),
        }
    }
}

pub(crate) fn select<R>(select: &Select<R>) -> String {
    let mut w = Writer::default();
    for (i, column) in select.columns.iter().enumerate() {
        w.sql.push_str(if i == 0 { " " } else { ", " });
        w.node(column);
    }

    w.sql.push_str(" FROM ");
    w.ident(select.from);
    w.sql
}
