//! Tables and handles.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

/// A table in the database.
pub trait Table: Copy + 'static {
    /// The table's name in the database.
    const NAME: &'static str;
}

/// Tells one use of a table in a statement from every other.
///
/// IDs are unique across the whole process, not just within one query.
/// Queries built separately end up in one statement as subqueries, so IDs
/// that restarted per query would collide: a handle used in the wrong query
/// could match a table there and render the wrong column instead of failing.
/// IDs never appear in the SQL, so the text stays the same from run to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceId(u64);

impl SourceId {
    fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        // A wrapped counter would hand out IDs already in use, so running out
        // panics instead, as [`std::thread::ThreadId`] does.
        NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map(Self)
            .expect("howdah ran out of table handle IDs")
    }
}

/// One use of the table `T` in a statement, made by [`from`](crate::from).
/// Its columns come from the table's column trait.
///
/// A handle has no methods of its own, so it cannot clash with a column name.
pub struct Handle<T> {
    source: SourceId,
    _table: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    pub(crate) fn new() -> Self {
        Self {
            source: SourceId::next(),
            _table: PhantomData,
        }
    }

    pub(crate) fn source(&self) -> SourceId {
        self.source
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}
