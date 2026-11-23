//! Typed Postgres queries for Rust.

mod error;
mod expr;
mod render;
mod select;
mod table;

pub use error::Error;
pub use expr::{Column, NeverNull, Nullable};
pub use select::{Select, Selection};
pub use table::{Handle, Table};

/// Starts a select from `table`. Returns the select and the handle of the
/// table, whose columns the select can then use.
pub fn from<T: Table>(_table: T) -> (Select, Handle<T>) {
    let handle = Handle::new();
    (Select::new(handle.source(), T::NAME), handle)
}
