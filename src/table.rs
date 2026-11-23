//! Tables and handles.

use std::marker::PhantomData;

/// A table in the database.
pub trait Table: Copy + 'static {
    /// The table's name in the database.
    const NAME: &'static str;
}

/// One use of the table `T` in a statement, made by [`from`](crate::from).
/// Its columns come from the table's column trait.
///
/// A handle has no methods of its own, so it cannot clash with a column name.
pub struct Handle<T> {
    _table: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    pub(crate) fn new() -> Self {
        Self {
            _table: PhantomData,
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}
