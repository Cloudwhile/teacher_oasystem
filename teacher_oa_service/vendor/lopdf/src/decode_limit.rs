//! TeacherOA patch: per-thread budget for decoded stream bytes.
//!
//! Upstream decoders inflate streams without bound while a document loads, so a small upload
//! can expand to gigabytes. Callers handling untrusted input wrap loading in [`with_limit`];
//! every decode then draws from one shared budget and fails once it is spent.
use std::cell::Cell;

thread_local! {
    static REMAINING: Cell<Option<usize>> = const { Cell::new(None) };
    static EXCEEDED: Cell<bool> = const { Cell::new(false) };
}

struct Restore(Option<usize>, bool);

impl Drop for Restore {
    fn drop(&mut self) {
        REMAINING.with(|r| r.set(self.0));
        EXCEEDED.with(|e| e.set(self.1));
    }
}

/// Runs `f` with at most `limit` decoded bytes available to all decoders on this thread.
/// Returns `None` when the budget was exhausted, even if `f` itself recovered from the error.
pub fn with_limit<T>(limit: usize, f: impl FnOnce() -> T) -> Option<T> {
    let _restore = Restore(REMAINING.with(|r| r.replace(Some(limit))), EXCEEDED.with(|e| e.replace(false)));
    let value = f();
    if EXCEEDED.with(Cell::get) { None } else { Some(value) }
}

/// Bytes still available, or `None` when no limit is active.
pub(crate) fn remaining() -> Option<usize> {
    REMAINING.with(Cell::get)
}

/// Consumes `n` bytes of the active budget.
pub(crate) fn charge(n: usize) -> crate::Result<()> {
    REMAINING.with(|r| match r.get() {
        None => Ok(()),
        Some(left) if n <= left => {
            r.set(Some(left - n));
            Ok(())
        }
        Some(_) => {
            r.set(Some(0));
            EXCEEDED.with(|e| e.set(true));
            Err(crate::error::DecompressError::LimitExceeded.into())
        }
    })
}
