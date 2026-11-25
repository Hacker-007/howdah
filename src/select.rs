//! Building a select.

use std::marker::PhantomData;

use crate::error::Error;
use crate::expression::{ErasedExpression, Expression};
use crate::format;
use crate::null::Nullability;
use crate::table::SourceId;
use crate::value::Value;

/// A select statement. `R` is the type of each row it returns: `()` until
/// [`select`](Select::select) chooses the columns.
///
/// A select is an ordinary value: it can be cloned, stored and returned.
pub struct Select<R = ()> {
    pub(crate) source: SourceId,
    pub(crate) from: &'static str,
    pub(crate) columns: Vec<ErasedExpression>,
    pub(crate) filters: Vec<ErasedExpression>,
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
    /// The select with these columns: a column or expression, or a tuple of
    /// them. The select it is called on stays as it is, so one select can give
    /// several.
    pub fn select<S: Selection>(&self, selection: S) -> Select<S::Row> {
        Select {
            source: self.source,
            from: self.from,
            columns: selection.components(),
            filters: self.filters.clone(),
            _row: PhantomData,
        }
    }

    /// Keeps only the rows where `condition` is true. A row where it is null
    /// is left out, as in SQL. Every filter must hold.
    pub fn filter<E: Expression<Type = bool>>(&mut self, condition: E) -> &mut Self {
        self.filters.push(condition.erase());
        self
    }

    /// The statement's SQL and the values of its parameters, in order: the
    /// first is `$1`.
    ///
    /// Fails if a column's table is not part of the select.
    pub fn to_sql(&self) -> Result<(String, Vec<Value>), Error> {
        format::select(self)
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

/// What a select returns for each row: a column or expression, or a tuple of
/// them.
pub trait Selection {
    /// The Rust type of a row.
    type Row;

    #[doc(hidden)]
    fn components(self) -> Vec<ErasedExpression>;
}

impl Selection for () {
    type Row = ();

    fn components(self) -> Vec<ErasedExpression> {
        vec![]
    }
}

impl<E: Expression> Selection for E {
    type Row = <E::Null as Nullability>::Read<E::Type>;

    fn components(self) -> Vec<ErasedExpression> {
        vec![self.erase()]
    }
}

macro_rules! tuples {
    ($(($($name:ident),+))*) => {
        $(
            impl<$($name: Selection),+> Selection for ($($name,)+) {
                type Row = ($($name::Row,)+);

                #[allow(non_snake_case)]
                fn components(self) -> Vec<ErasedExpression> {
                    let mut components = vec![];
                    let ($($name,)+) = self;
                    $(components.extend($name.components());)+
                    components
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
