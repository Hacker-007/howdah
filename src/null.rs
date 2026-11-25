//! Whether a value can be null.

/// The value can never be null.
pub struct NeverNull;

/// The value may be null.
pub struct Nullable;

/// Whether a value can be null: [`NeverNull`] or [`Nullable`].
pub trait Nullability {
    /// Nullable if either `Self` or `M` is.
    type Or<M: Nullability>: Nullability;
}

impl Nullability for NeverNull {
    type Or<M: Nullability> = M;
}

impl Nullability for Nullable {
    type Or<M: Nullability> = Nullable;
}
