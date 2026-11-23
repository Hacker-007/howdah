//! Building a select.

use std::marker::PhantomData;

use crate::error::Error;
use crate::expr::{Column, NeverNull, Node, Nullable};
use crate::render;
use crate::table::SourceId;

/// A select statement. `R` is the type of each row it returns: `()` until
/// [`select`](Select::select) chooses the columns.
///
/// A select is an ordinary value: it can be cloned, stored and returned.
pub struct Select<R = ()> {
    pub(crate) source: SourceId,
    pub(crate) from: &'static str,
    pub(crate) columns: Vec<Node>,
    pub(crate) filters: Vec<Node>,
    _row: PhantomData<fn() -> R>,
}

impl Select {
    pub(crate) fn new(source: SourceId, from: &'static str) -> Self {
        Self {
            source,
            from,
            columns: Vec::new(),
            filters: Vec::new(),
            _row: PhantomData,
        }
    }
}

impl<R> Select<R> {
    /// The select with these columns: one column, or a tuple of them. The
    /// select it is called on stays as it is, so one select can give several.
    pub fn select<S: Selection>(&self, columns: S) -> Select<S::Row> {
        let mut nodes = Vec::new();
        columns.push_columns(&mut nodes);

        Select {
            source: self.source,
            from: self.from,
            columns: nodes,
            filters: self.filters.clone(),
            _row: PhantomData,
        }
    }

    /// Keeps only the rows where `condition` is true. A row where it is null
    /// is left out, as in SQL. Every filter must hold.
    pub fn filter<N>(&mut self, condition: Column<bool, N>) -> &mut Self {
        self.filters.push(condition.node());
        self
    }

    /// The statement's SQL.
    ///
    /// Fails if a column's table is not part of the select.
    pub fn to_sql(&self) -> Result<String, Error> {
        render::select(self)
    }
}

impl<R> Clone for Select<R> {
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            from: self.from,
            columns: self.columns.clone(),
            filters: self.filters.clone(),
            _row: PhantomData,
        }
    }
}

/// What a select returns for each row: one column, or a tuple of them.
pub trait Selection {
    /// The Rust type of a row.
    type Row;

    #[doc(hidden)]
    fn push_columns(self, into: &mut Vec<Node>);
}

impl<T> Selection for Column<T, NeverNull> {
    type Row = T;

    fn push_columns(self, into: &mut Vec<Node>) {
        into.push(self.node());
    }
}

impl<T> Selection for Column<T, Nullable> {
    type Row = Option<T>;

    fn push_columns(self, into: &mut Vec<Node>) {
        into.push(self.node());
    }
}

macro_rules! tuples {
    ($(($($name:ident),+))*) => {
        $(
            impl<$($name: Selection),+> Selection for ($($name,)+) {
                type Row = ($($name::Row,)+);

                #[allow(non_snake_case)]
                fn push_columns(self, into: &mut Vec<Node>) {
                    let ($($name,)+) = self;
                    $($name.push_columns(into);)+
                }
            }
        )*
    };
}

tuples! {
    (A)
    (A, B)
    (A, B, C)
    (A, B, C, D)
    (A, B, C, D, E)
    (A, B, C, D, E, F)
    (A, B, C, D, E, F, G)
    (A, B, C, D, E, F, G, H)
    (A, B, C, D, E, F, G, H, I)
    (A, B, C, D, E, F, G, H, I, J)
    (A, B, C, D, E, F, G, H, I, J, K)
    (A, B, C, D, E, F, G, H, I, J, K, L)
}
