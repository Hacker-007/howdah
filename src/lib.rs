//! Typed Postgres queries for Rust.

mod error;
mod expression;
mod format;
mod null;
mod select;
mod table;
mod value;

pub use error::Error;
pub use expression::{Column, Expr, Expression, IntoExpression, Order};
pub use null::{NeverNull, Nullability, Nullable};
pub use select::{Select, Selection};
pub use table::{Handle, Table};
pub use value::Value;

/// Starts a select from `table`. Returns the select and the handle of the
/// table, whose columns the select can then use.
pub fn from<T: Table>(_table: T) -> (Select, Handle<T>) {
    let handle = Handle::new();
    (Select::new(handle.source(), T::NAME), handle)
}
