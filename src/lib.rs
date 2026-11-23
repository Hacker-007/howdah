//! Typed Postgres queries for Rust.

mod expr;
mod render;
mod select;
mod table;

pub use expr::Column;
pub use select::{Select, Selection};
pub use table::{Handle, Table};

/// Starts a select from `table`. Returns the select and the handle of the
/// table, whose columns the select can then use.
pub fn from<T: Table>(_table: T) -> (Select, Handle<T>) {
    (Select::new(T::NAME), Handle::new())
}
