//! Columns, expressions and the untyped tree they become.

use std::marker::PhantomData;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Not, Rem, Sub};

use crate::null::{NeverNull, Nullability};
use crate::table::{Handle, SourceId, Table};
use crate::value::Value;

/// An expression holding values of type `T`, such as a comparison. `N` is
/// [`Nullable`](crate::Nullable) if it can be null.
///
/// Arithmetic operators work on numeric columns and expressions. Both sides
/// must have the same type:
///
/// ```compile_fail,E0277
/// # use howdah::{Column, Table, from};
/// # #[derive(Clone, Copy)]
/// # struct Users;
/// # impl Table for Users {
/// #     const NAME: &'static str = "users";
/// # }
/// let (_, u) = from(Users);
/// let id: Column<i64> = Column::new(u, "id");
/// let small: Column<i32> = Column::new(u, "small");
/// let _ = id + small;
/// ```
///
/// `%` needs integers, as in Postgres:
///
/// ```compile_fail,E0277
/// # use howdah::{Column, Table, from};
/// # #[derive(Clone, Copy)]
/// # struct Users;
/// # impl Table for Users {
/// #     const NAME: &'static str = "users";
/// # }
/// let (_, u) = from(Users);
/// let total: Column<f64> = Column::new(u, "total");
/// let _ = total % 2.0;
/// ```
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
    Logic {
        op: LogicOperation,
        left: Box<ErasedExpression>,
        right: Box<ErasedExpression>,
    },
    Not(Box<ErasedExpression>),
    IsNull(Box<ErasedExpression>),
    IsNotNull(Box<ErasedExpression>),
    Arithmetic {
        op: ArithmeticOperation,
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
    pub fn new<S: Table, M>(handle: Handle<S, M>, name: &'static str) -> Self {
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

/// A logical operator joining two boolean expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicOperation {
    And,
    Or,
}

/// An arithmetic operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

/// Which way to order rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Asc,
    Desc,
}

/// An expression to order rows by, with its direction. Made by
/// [`Expression::asc`] and [`Expression::desc`].
#[derive(Debug, Clone)]
pub struct Order {
    pub(crate) expression: ErasedExpression,
    pub(crate) direction: Direction,
}

/// What `+` works on: the numeric types. Dates and intervals will join it.
pub trait Additive {}

/// What `-`, `*` and `/` work on: the numeric types.
pub trait Numeric {}

/// What `%` works on: the integer types. Postgres has no `%` for reals.
pub trait Remainder {}

macro_rules! markers {
    ($($marker:ident: $($type:ty),*;)*) => {
        $($(
            impl $marker for $type {}
        )*)*
    };
}

markers! {
    Additive: i16, i32, i64, f32, f64;
    Numeric: i16, i32, i64, f32, f64;
    Remainder: i16, i32, i64;
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
///
/// `&`, `|` and `!` are SQL's `AND`, `OR` and `NOT`, and work only on boolean
/// expressions:
///
/// ```compile_fail,E0369
/// # use howdah::{Column, Table, from};
/// # #[derive(Clone, Copy)]
/// # struct Users;
/// # impl Table for Users {
/// #     const NAME: &'static str = "users";
/// # }
/// let (_, u) = from(Users);
/// let id: Column<i64> = Column::new(u, "id");
/// let active: Column<bool> = Column::new(u, "active");
/// let _ = id & active;
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

    /// Order rows by this expression, ascending. Used with
    /// [`Select::order_by`](crate::Select::order_by).
    fn asc(self) -> Order {
        Order {
            expression: self.erase(),
            direction: Direction::Asc,
        }
    }

    /// Order rows by this expression, descending. Used with
    /// [`Select::order_by`](crate::Select::order_by).
    fn desc(self) -> Order {
        Order {
            expression: self.erase(),
            direction: Direction::Desc,
        }
    }

    /// `self IS NULL`. Never null itself.
    #[expect(
        clippy::wrong_self_convention,
        reason = "builds SQL like the other methods; it does not test `self`"
    )]
    fn is_null(self) -> Expr<bool> {
        Expr::new(ErasedExpression::IsNull(Box::new(self.erase())))
    }

    /// `self IS NOT NULL`. Never null itself.
    #[expect(
        clippy::wrong_self_convention,
        reason = "builds SQL like the other methods; it does not test `self`"
    )]
    fn is_not_null(self) -> Expr<bool> {
        Expr::new(ErasedExpression::IsNotNull(Box::new(self.erase())))
    }
}

/// `&`, `|` and `!` on boolean columns and expressions. The result is
/// nullable if an operand is.
macro_rules! logic_operators {
    ($($type:ident),*) => {
        $(
            /// `self AND other`.
            impl<N: Nullability, E: IntoExpression<bool>> BitAnd<E> for $type<bool, N> {
                type Output = Expr<bool, N::Or<E::Null>>;

                fn bitand(self, other: E) -> Self::Output {
                    Expr::new(ErasedExpression::Logic {
                        op: LogicOperation::And,
                        left: Box::new(self.erase()),
                        right: Box::new(other.erase()),
                    })
                }
            }

            /// `self OR other`.
            impl<N: Nullability, E: IntoExpression<bool>> BitOr<E> for $type<bool, N> {
                type Output = Expr<bool, N::Or<E::Null>>;

                fn bitor(self, other: E) -> Self::Output {
                    Expr::new(ErasedExpression::Logic {
                        op: LogicOperation::Or,
                        left: Box::new(self.erase()),
                        right: Box::new(other.erase()),
                    })
                }
            }

            /// `NOT self`.
            impl<N: Nullability> Not for $type<bool, N> {
                type Output = Expr<bool, N>;

                fn not(self) -> Self::Output {
                    Expr::new(ErasedExpression::Not(Box::new(self.erase())))
                }
            }
        )*
    };
}

logic_operators!(Column, Expr);

/// `+` on [`Additive`] columns and expressions, `-`, `*` and `/` on
/// [`Numeric`] ones and `%` on [`Remainder`] ones. Both sides must have the
/// same type. The result is nullable if an operand is.
macro_rules! arithmetic {
    ($($op:ident :: $method:ident => $operation:ident, $marker:ident);* $(;)?) => {
        $(
            impl<T: $marker, N: Nullability, E: IntoExpression<T>> $op<E> for Column<T, N> {
                type Output = Expr<T, N::Or<E::Null>>;

                fn $method(self, other: E) -> Self::Output {
                    Expr::new(ErasedExpression::Arithmetic {
                        op: ArithmeticOperation::$operation,
                        left: Box::new(self.erase()),
                        right: Box::new(other.erase()),
                    })
                }
            }

            impl<T: $marker, N: Nullability, E: IntoExpression<T>> $op<E> for Expr<T, N> {
                type Output = Expr<T, N::Or<E::Null>>;

                fn $method(self, other: E) -> Self::Output {
                    Expr::new(ErasedExpression::Arithmetic {
                        op: ArithmeticOperation::$operation,
                        left: Box::new(self.erase()),
                        right: Box::new(other.erase()),
                    })
                }
            }
        )*
    };
}

arithmetic! {
    Add::add => Add, Additive;
    Sub::sub => Subtract, Numeric;
    Mul::mul => Multiply, Numeric;
    Div::div => Divide, Numeric;
    Rem::rem => Remainder, Remainder;
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
