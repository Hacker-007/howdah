//! Rust values sent with a statement as parameters.

use crate::expression::{Erase, ErasedExpression, IntoExpression};
use crate::null::NeverNull;

/// A value sent with a statement as a parameter, such as `$1`.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Value {
    Boolean(bool),
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Real(f32),
    DoublePrecision(f64),
    Text(String),
}

macro_rules! literals {
    ($($type:ty => $variant:ident),* $(,)?) => {
        $(
            impl Erase for $type {
                fn erase(self) -> ErasedExpression {
                    ErasedExpression::Value(Value::$variant(self))
                }
            }

            impl IntoExpression<$type> for $type {
                type Null = NeverNull;
            }
        )*
    };
}

literals! {
    bool => Boolean,
    i16 => SmallInt,
    i32 => Integer,
    i64 => BigInt,
    f32 => Real,
    f64 => DoublePrecision,
    String => Text,
}

impl Erase for &str {
    fn erase(self) -> ErasedExpression {
        ErasedExpression::Value(Value::Text(self.to_owned()))
    }
}

impl IntoExpression<String> for &str {
    type Null = NeverNull;
}
