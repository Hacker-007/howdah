//! Whether a value can be null.

/// The value can never be null.
pub struct NeverNull;

/// The value may be null.
pub struct Nullable;

/// Whether a value can be null: [`NeverNull`] or [`Nullable`].
pub trait Nullability {
    /// Nullable if either `Self` or `M` is.
    type Or<M: Nullability>: Nullability;

    /// How a value of type `T` is read from a row: `T`, or `Option<T>` if it
    /// can be null.
    type Read<T>;
}

impl Nullability for NeverNull {
    type Or<M: Nullability> = M;
    type Read<T> = T;
}

impl Nullability for Nullable {
    type Or<M: Nullability> = Nullable;
    type Read<T> = Option<T>;
}
