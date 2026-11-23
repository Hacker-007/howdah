//! Mistakes found when a statement is rendered.

use std::fmt;

/// A statement that cannot be rendered. Nothing is sent to the database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A column used where its handle's table is not part of the statement,
    /// such as a column from another select.
    NotInScope {
        table: &'static str,
        column: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInScope { table, column } => write!(
                f,
                "`{table}.{column}` is used where its table is not part of the statement"
            ),
        }
    }
}

impl std::error::Error for Error {}
