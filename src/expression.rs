//! Columns, expressions and the untyped tree they become.

use std::marker::PhantomData;

use crate::null::{NeverNull, Nullability};
use crate::table::{Handle, SourceId, Table};

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
///
/// It is `pub` only so that public traits such as [`Expression`] can name it.
/// This module is private, so no other crate can.
#[derive(Debug, Clone)]
pub enum ErasedExpression {
    Column(ColumnRef),
    Compare {
        op: CompareOperation,
        left: Box<ErasedExpression>,
        right: Box<ErasedExpression>,
    },
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

/// A column with its type removed. `pub` for the same reason as
/// [`ErasedExpression`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnRef {
    pub(crate) source: SourceId,
    pub(crate) table: &'static str,
    pub(crate) name: &'static str,
}

/// A comparison operator. `pub` for the same reason as [`ErasedExpression`].
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
/// ```compile_fail,E0271
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
pub trait Expression: Sized {
    /// The Rust type of the value.
    type Type;

    /// Whether the value can be null.
    type Null: Nullability;

    #[doc(hidden)]
    fn erase(self) -> ErasedExpression;

    /// `self = other`.
    fn eq<E>(self, other: E) -> Expr<bool, <Self::Null as Nullability>::Or<E::Null>>
    where
        E: Expression<Type = Self::Type>,
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
        E: Expression<Type = Self::Type>,
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
        E: Expression<Type = Self::Type>,
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
        E: Expression<Type = Self::Type>,
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
        E: Expression<Type = Self::Type>,
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
        E: Expression<Type = Self::Type>,
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

    fn erase(self) -> ErasedExpression {
        ErasedExpression::Column(self.column)
    }
}

impl<T, N: Nullability> Expression for Expr<T, N> {
    type Type = T;
    type Null = N;

    fn erase(self) -> ErasedExpression {
        self.inner
    }
}
