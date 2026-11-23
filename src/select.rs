//! Building a select.

use std::marker::PhantomData;

use crate::expr::{Column, Node};
use crate::render;

#[derive(Debug, Clone)]
pub(crate) struct SelectAst {
    pub(crate) from: &'static str,
    pub(crate) columns: Vec<Node>,
}

/// A select being built. Make one with [`from`].
///
/// A query is an ordinary value: it can be cloned, stored and returned.
#[derive(Debug, Clone)]
pub struct Query {
    pub(crate) ast: SelectAst,
}

impl Query {
    /// The query with this column. The query stays usable, so one query can
    /// give several selects.
    pub fn select<T>(&self, column: Column<T>) -> Select<T> {
        let mut ast = self.ast.clone();
        ast.columns = vec![column.node()];
        Select {
            ast,
            _row: PhantomData,
        }
    }
}

/// A select statement. `R` is the type of each row it returns.
pub struct Select<R> {
    ast: SelectAst,
    _row: PhantomData<fn() -> R>,
}

impl<R> Clone for Select<R> {
    fn clone(&self) -> Self {
        Self {
            ast: self.ast.clone(),
            _row: PhantomData,
        }
    }
}

impl<R> Select<R> {
    /// The statement's SQL.
    pub fn to_sql(&self) -> String {
        render::select(&self.ast)
    }
}
