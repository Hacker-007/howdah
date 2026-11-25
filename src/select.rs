//! Building a select.

use std::marker::PhantomData;

use crate::error::Error;
use crate::expression::{ErasedExpression, Expression, Order};
use crate::format;
use crate::null::{Nullability, Nullable};
use crate::table::{Handle, SourceId, Table};
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
    pub(crate) orders: Vec<Order>,
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
    pub(crate) joins: Vec<Join>,
    _row: PhantomData<fn() -> R>,
}

/// A join: another use of a table, and the condition rows must match.
#[derive(Debug, Clone)]
pub(crate) struct Join {
    pub(crate) source: SourceId,
    pub(crate) table: &'static str,
    pub(crate) kind: JoinKind,
    pub(crate) on: ErasedExpression,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum JoinKind {
    Inner,
    Left,
}

impl Select {
    pub(crate) fn new(source: SourceId, from: &'static str) -> Self {
        Self {
            source,
            from,
            columns: Vec::new(),
            filters: Vec::new(),
            orders: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
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
            orders: self.orders.clone(),
            limit: self.limit,
            offset: self.offset,
            joins: self.joins.clone(),
            _row: PhantomData,
        }
    }

    /// Keeps only the rows where `condition` is true. A row where it is null
    /// is left out, as in SQL. Every filter must hold.
    pub fn filter<E: Expression<Type = bool>>(&mut self, condition: E) -> &mut Self {
        self.filters.push(condition.erase());
        self
    }

    /// Orders rows by this expression, then by each later `order_by`:
    /// `q.order_by(u.name().asc())`.
    pub fn order_by(&mut self, order: Order) -> &mut Self {
        self.orders.push(order);
        self
    }

    /// Returns at most `limit` rows. Calling it again replaces the earlier
    /// limit.
    pub fn limit(&mut self, limit: u64) -> &mut Self {
        self.limit = Some(limit);
        self
    }

    /// Skips `offset` rows. Calling it again replaces the earlier offset.
    pub fn offset(&mut self, offset: u64) -> &mut Self {
        self.offset = Some(offset);
        self
    }

    /// Joins `table`, keeping only rows that match `on`. The condition may use
    /// the handles of the select and of every earlier join. Returns the
    /// handle of this use of the table, so the same table can be joined more
    /// than once.
    pub fn join<T: Table, E: Expression<Type = bool>>(
        &mut self,
        table: T,
        on: impl FnOnce(Handle<T>) -> E,
    ) -> Handle<T> {
        self.join_with(table, JoinKind::Inner, on)
    }

    /// Left-joins `table`, keeping every row whether or not it matches `on`.
    /// Columns read through the returned handle are nullable.
    pub fn left_join<T: Table, E: Expression<Type = bool>>(
        &mut self,
        table: T,
        on: impl FnOnce(Handle<T, Nullable>) -> E,
    ) -> Handle<T, Nullable> {
        self.join_with(table, JoinKind::Left, on)
    }

    fn join_with<T: Table, N: Nullability, E: Expression<Type = bool>>(
        &mut self,
        _table: T,
        kind: JoinKind,
        on: impl FnOnce(Handle<T, N>) -> E,
    ) -> Handle<T, N> {
        let handle = Handle::new();
        self.joins.push(Join {
            source: handle.source(),
            table: T::NAME,
            kind,
            on: on(handle).erase(),
        });
        handle
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
            orders: self.orders.clone(),
            limit: self.limit,
            offset: self.offset,
            joins: self.joins.clone(),
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
