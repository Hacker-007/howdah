//! Columns and the untyped tree they become.

use std::marker::PhantomData;

use crate::table::{Handle, Table};

/// A column with its type removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ColumnRef {
    pub(crate) table: &'static str,
    pub(crate) name: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) enum Node {
    Column(ColumnRef),
}

/// A column of one [`Handle`], holding values of type `T`.
pub struct Column<T> {
    column: ColumnRef,
    _type: PhantomData<fn() -> T>,
}

impl<T> Column<T> {
    #[doc(hidden)]
    pub fn new<S: Table>(_handle: Handle<S>, name: &'static str) -> Self {
        Self {
            column: ColumnRef {
                table: S::NAME,
                name,
            },
            _type: PhantomData,
        }
    }

    pub(crate) fn node(self) -> Node {
        Node::Column(self.column)
    }
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Column<T> {}
