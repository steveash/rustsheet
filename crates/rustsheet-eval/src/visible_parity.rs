//! The visible-parity ratchet (T-0061, decision 0088): the in-scope cells of the vendored gated
//! suites that are visibly off from Excel (decision 0083) may not grow. `rustsheet-eval check`
//! holds a full gated run to [`LOCK_PATH`] with [`judge`]; `update-baseline` records it next to
//! the baselines.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::closeness::Visible;
use crate::report::{calls_only_in_scope, functions_in};
use crate::runner::CaseResult;

/// The committed lock, relative to the repository root. It sits with the baselines because
/// `make eval-baseline` records it with them, and like them it is never hand-edited.
pub const LOCK_PATH: &str = "eval/baseline/visible-parity.json";

/// The committed lock.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    /// In-scope cells of the vendored gated suites that are visibly off.
    pub visibly_off_cells: usize,
}

/// Failure to read or write the lock.
#[derive(Debug, thiserror::Error)]
pub enum LockError {
    /// The lock file could not be read or written.
    #[error("{path}: {source}")]
    Io {
        /// The file that failed.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
    /// The lock is not valid JSON of the expected shape.
    #[error("parsing {LOCK_PATH}: {0}")]
    Parse(#[from] serde_json::Error),
}

impl Lock {
    /// Reads [`LOCK_PATH`] under `root`. A missing lock allows no visibly-off cell.
    ///
    /// # Errors
    /// When the file cannot be read or is malformed.
    pub fn read(root: &Path) -> Result<Lock, LockError> {
        let path = root.join(LOCK_PATH);
        match std::fs::read_to_string(&path) {
            Ok(text) => Ok(serde_json::from_str(&text)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Lock {
                visibly_off_cells: 0,
            }),
            Err(source) => Err(LockError::Io { path, source }),
        }
    }

    /// Writes the lock to [`LOCK_PATH`] under `root`.
    ///
    /// # Errors
    /// When the file cannot be written.
    pub fn write(&self, root: &Path) -> Result<(), LockError> {
        let path = root.join(LOCK_PATH);
        let text = format!("{}\n", serde_json::to_string_pretty(self)?);
        std::fs::write(&path, text).map_err(|source| LockError::Io { path, source })
    }
}

/// The visibly-off in-scope cells of a run, and how many of them are in cases the committed
/// baseline does not know yet (new coverage, which `update-baseline` may record as a rise).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VisibleCount {
    /// Every visibly-off in-scope cell.
    pub cells: usize,
    /// Of those, the cells of cases new to the baseline.
    pub new_coverage: usize,
}

impl VisibleCount {
    /// Counts `case`'s visibly-off cells that call no out-of-scope function; `new_case` when the
    /// committed baseline does not list the case.
    pub fn add(&mut self, catalog: &Catalog, case: &CaseResult, new_case: bool) {
        let n = case
            .cells
            .iter()
            .filter(|c| Visible::of(c).total() > 0)
            .filter(|c| {
                let called = c.formula.as_deref().map(functions_in).unwrap_or_default();
                calls_only_in_scope(catalog, &called)
            })
            .count();
        self.cells += n;
        if new_case {
            self.new_coverage += n;
        }
    }
}

/// What the gate decided about a run's count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judgement {
    /// The count equals the lock.
    Unchanged,
    /// Record the count (`update-baseline` only).
    Record,
    /// Fewer cells are visibly off than the lock says: run `make eval-baseline`.
    Stale,
    /// More cells are visibly off than the lock (plus, when recording, the new coverage) allows.
    Regressed,
}

/// Judges `count` against `lock`. `update` is `None` for `check` and `Some(allow_regressions)`
/// for `update-baseline`, which records a drop, a rise that new cases account for, and any rise
/// with `--allow-regressions` (which needs a decision record).
#[must_use]
pub fn judge(lock: Lock, count: VisibleCount, update: Option<bool>) -> Judgement {
    let locked = lock.visibly_off_cells;
    match update {
        _ if count.cells == locked => Judgement::Unchanged,
        None if count.cells < locked => Judgement::Stale,
        Some(allow) if allow || count.cells <= locked + count.new_coverage => Judgement::Record,
        None | Some(_) => Judgement::Regressed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(cells: usize, new_coverage: usize) -> VisibleCount {
        VisibleCount {
            cells,
            new_coverage,
        }
    }

    #[test]
    fn check_holds_the_count_to_the_lock() {
        let lock = Lock {
            visibly_off_cells: 10,
        };
        assert_eq!(judge(lock, count(10, 0), None), Judgement::Unchanged);
        assert_eq!(judge(lock, count(9, 0), None), Judgement::Stale);
        assert_eq!(judge(lock, count(11, 5), None), Judgement::Regressed);
    }

    #[test]
    fn update_records_drops_and_new_coverage_but_not_worse_cells() {
        let lock = Lock {
            visibly_off_cells: 10,
        };
        assert_eq!(judge(lock, count(7, 0), Some(false)), Judgement::Record);
        assert_eq!(judge(lock, count(13, 3), Some(false)), Judgement::Record);
        assert_eq!(judge(lock, count(13, 2), Some(false)), Judgement::Regressed);
        assert_eq!(judge(lock, count(13, 0), Some(true)), Judgement::Record);
        assert_eq!(judge(lock, count(10, 4), Some(false)), Judgement::Unchanged);
    }

    #[test]
    fn lock_rejects_unknown_fields_and_the_committed_lock_parses() {
        assert!(serde_json::from_str::<Lock>(r#"{"visibly_off_cells": 1, "x": 2}"#).is_err());
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(Lock::read(&root).is_ok());
    }
}
