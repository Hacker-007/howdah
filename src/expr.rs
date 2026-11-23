//! Columns and the untyped tree they become.

use std::marker::PhantomData;

use crate::table::{Handle, SourceId, Table};

/// A column with its type removed. `pub` for the same reason as [`Node`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnRef {
    pub(crate) source: SourceId,
    pub(crate) table: &'static str,
    pub(crate) name: &'static str,
}

/// An expression with its types removed.
///
/// It is `pub` only so that the public [`Selection`](crate::Selection) trait
/// can name it. This module is private, so no other crate can.
#[derive(Debug, Clone)]
pub enum Node {
    Column(ColumnRef),
}

/// The value can never be null.
pub struct NeverNull;

/// The value may be null.
pub struct Nullable;

/// A column of one [`Handle`], holding values of type `T`. `N` is [`Nullable`]
/// for a column that can hold null.
pub struct Column<T, N = NeverNull> {
    column: ColumnRef,
    _type: PhantomData<fn() -> (T, N)>,
}

impl<T, N> Column<T, N> {
    #[doc(hidden)]
    pub fn new<S: Table>(handle: Handle<S>, name: &'static str) -> Self {
        Self {
            column: ColumnRef {
                source: handle.source(),
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

impl<T, N> Clone for Column<T, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, N> Copy for Column<T, N> {}
