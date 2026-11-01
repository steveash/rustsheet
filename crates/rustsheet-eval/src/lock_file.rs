//! Reading and writing the small JSON ratchet locks the gates commit (the evidence floor,
//! decision 0062), and the error type the line-oriented visible-parity lock shares with them
//! (decisions 0088 and 0219).

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Failure to read or write a lock.
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
    #[error("parsing {path}: {source}")]
    Parse {
        /// The lock's path relative to the repository root.
        path: &'static str,
        /// The underlying error.
        source: serde_json::Error,
    },
    /// A line of a line-oriented lock (visible parity, decision 0219) is malformed or repeats a
    /// case.
    #[error("{path}:{line}: malformed line (expected `case<TAB>count`, each case once)")]
    Line {
        /// The lock's path relative to the repository root.
        path: &'static str,
        /// The 1-based line number.
        line: usize,
    },
}

/// Reads the lock at `rel` under `root`, or `missing` when there is no such file.
///
/// # Errors
/// When the file cannot be read or is malformed.
pub fn read<T: DeserializeOwned>(
    root: &Path,
    rel: &'static str,
    missing: T,
) -> Result<T, LockError> {
    let path = root.join(rel);
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            serde_json::from_str(&text).map_err(|source| LockError::Parse { path: rel, source })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(missing),
        Err(source) => Err(LockError::Io { path, source }),
    }
}

/// Writes `lock` as pretty JSON to `rel` under `root`.
///
/// # Errors
/// When the file cannot be written.
pub fn write<T: Serialize>(root: &Path, rel: &'static str, lock: &T) -> Result<(), LockError> {
    let path = root.join(rel);
    let json = serde_json::to_string_pretty(lock)
        .map_err(|source| LockError::Parse { path: rel, source })?;
    std::fs::write(&path, format!("{json}\n")).map_err(|source| LockError::Io { path, source })
}
