//! Columns, expressions and the untyped tree they become.

use std::marker::PhantomData;

use crate::null::{NeverNull, Nullability};
use crate::table::{Handle, SourceId, Table};
use crate::value::Value;

/// An expression holding values of type `T`, such as a comparison. `N` is
/// [`Nullable`](crate::Nullable) if it can be null.
pub struct Expr<T, N = NeverNull> {
    inner: ErasedExpression,
    _type: PhantomData<fn() -> (T, N)>,
}

impl<T, N> Expr<T, N> {
    fn new(inner: ErasedExpression) -> Self {
        Self {
            inner,
            _type: PhantomData,
        }
    }
}

impl<T, N> Clone for Expr<T, N> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

/// An expression with its types removed.
#[derive(Debug, Clone)]
pub enum ErasedExpression {
    Column(ColumnRef),
    Compare {
        op: CompareOperation,
        left: Box<ErasedExpression>,
        right: Box<ErasedExpression>,
    },
    Value(Value),
}

/// A column of one [`Handle`], holding values of type `T`. `N` is
/// [`Nullable`](crate::Nullable) for a column that can hold null.
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
}

impl<T, N> Clone for Column<T, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, N> Copy for Column<T, N> {}

/// A column with its type removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnRef {
    pub(crate) source: SourceId,
    pub(crate) table: &'static str,
    pub(crate) name: &'static str,
}

/// A comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOperation {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
}

/// A value in SQL: a column, or an expression built from them. Values of type
/// `Type` that are null when `Null` is [`Nullable`](crate::Nullable).
///
/// Both sides of a comparison must have the same `Type`:
///
/// ```compile_fail,E0277
/// # use howdah::{Column, Expression, Table, from};
/// # #[derive(Clone, Copy)]
/// # struct Users;
/// # impl Table for Users {
/// #     const NAME: &'static str = "users";
/// # }
/// let (_, u) = from(Users);
/// let id: Column<i64> = Column::new(u, "id");
/// let name: Column<String> = Column::new(u, "name");
/// id.eq(name);
/// ```
pub trait Expression: Erase + Sized {
    /// The Rust type of the value.
    type Type;

    /// Whether the value can be null.
    type Null: Nullability;

    /// `self = other`.
    fn eq<E>(self, other: E) -> Expr<bool, <Self::Null as Nullability>::Or<E::Null>>
    where
        E: IntoExpression<Self::Type>,
    {
        Expr::new(ErasedExpression::Compare {
            op: CompareOperation::Equal,
            left: Box::new(self.erase()),
            right: Box::new(other.erase()),
        })
    }

    /// `self <> other`.
    fn ne<E>(self, other: E) -> Expr<bool, <Self::Null as Nullability>::Or<E::Null>>
    where
        E: IntoExpression<Self::Type>,
    {
        Expr::new(ErasedExpression::Compare {
            op: CompareOperation::NotEqual,
            left: Box::new(self.erase()),
            right: Box::new(other.erase()),
        })
    }

    /// `self < other`.
    fn lt<E>(self, other: E) -> Expr<bool, <Self::Null as Nullability>::Or<E::Null>>
    where
        E: IntoExpression<Self::Type>,
    {
        Expr::new(ErasedExpression::Compare {
            op: CompareOperation::LessThan,
            left: Box::new(self.erase()),
            right: Box::new(other.erase()),
        })
    }

    /// `self <= other`.
    fn le<E>(self, other: E) -> Expr<bool, <Self::Null as Nullability>::Or<E::Null>>
    where
        E: IntoExpression<Self::Type>,
    {
        Expr::new(ErasedExpression::Compare {
            op: CompareOperation::LessThanEqual,
            left: Box::new(self.erase()),
            right: Box::new(other.erase()),
        })
    }

    /// `self > other`.
    fn gt<E>(self, other: E) -> Expr<bool, <Self::Null as Nullability>::Or<E::Null>>
    where
        E: IntoExpression<Self::Type>,
    {
        Expr::new(ErasedExpression::Compare {
            op: CompareOperation::GreaterThan,
            left: Box::new(self.erase()),
            right: Box::new(other.erase()),
        })
    }

    /// `self >= other`.
    fn ge<E>(self, other: E) -> Expr<bool, <Self::Null as Nullability>::Or<E::Null>>
    where
        E: IntoExpression<Self::Type>,
    {
        Expr::new(ErasedExpression::Compare {
            op: CompareOperation::GreaterThanEqual,
            left: Box::new(self.erase()),
            right: Box::new(other.erase()),
        })
    }
}

impl<T, N: Nullability> Expression for Column<T, N> {
    type Type = T;
    type Null = N;
}

impl<T, N: Nullability> Expression for Expr<T, N> {
    type Type = T;
    type Null = N;
}

/// What can stand where an expression of type `T` is expected: a column or
/// expression of type `T`, or a Rust value, which is sent as a parameter.
/// An integer literal becomes whichever integer type `T` is.
///
/// A Rust value must have the same type as the other side:
///
/// ```compile_fail,E0277
/// # use howdah::{Column, Expression, Table, from};
/// # #[derive(Clone, Copy)]
/// # struct Users;
/// # impl Table for Users {
/// #     const NAME: &'static str = "users";
/// # }
/// let (_, u) = from(Users);
/// let id: Column<i64> = Column::new(u, "id");
/// id.eq("1");
/// ```
pub trait IntoExpression<T>: Erase {
    /// Whether the value can be null.
    type Null: Nullability;
}

impl<E: Expression> IntoExpression<E::Type> for E {
    type Null = E::Null;
}

/// Turns a typed value into the untyped tree. No other crate can implement it, so none
/// can implement [`Expression`] or [`IntoExpression`] either.
pub trait Erase {
    #[doc(hidden)]
    fn erase(self) -> ErasedExpression;
}

impl<T, N> Erase for Column<T, N> {
    fn erase(self) -> ErasedExpression {
        ErasedExpression::Column(self.column)
    }
}

impl<T, N> Erase for Expr<T, N> {
    fn erase(self) -> ErasedExpression {
        self.inner
    }
}
