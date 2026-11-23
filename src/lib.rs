//! Typed Postgres queries for Rust.

mod expr;
mod render;
mod select;
mod table;

pub use expr::Column;
pub use select::{Query, Select};
pub use table::{Handle, Table};

use crate::select::SelectAst;

/// Starts a select from `table`. Returns the query and the handle of the
/// table, whose columns the query can then use.
pub fn from<T: Table>(_table: T) -> (Query, Handle<T>) {
    let query = Query {
        ast: SelectAst {
            from: T::NAME,
            columns: Vec::new(),
        },
    };

    (query, Handle::new())
}
