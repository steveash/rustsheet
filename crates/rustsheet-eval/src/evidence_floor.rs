//! The evidence floor (T-0056): every in-scope catalog function the oracle can evaluate should
//! have at least [`EVIDENCE_FLOOR`] cells with Excel-sourced expectations. The report lists the
//! functions below it by category, and `quality/evidence-floor.json` ratchets their number down
//! (decision 0062): `rustsheet-eval evidence-floor` fails when it rises or when the lock is stale,
//! and `--update` records a drop.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::report::FunctionEvidence;

/// Fewer Excel-evidence cells than this and a function's pass rate says little (T-0020); the
/// owner set the floor at 20 (T-0056).
pub const EVIDENCE_FLOOR: usize = 20;

/// Path of the ratchet lock, relative to the repository root.
pub const LOCK_PATH: &str = "quality/evidence-floor.json";

/// An in-scope, oracle-evaluable catalog function with fewer than [`EVIDENCE_FLOOR`] Excel cells.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BelowFloor {
    /// Upper-case Excel name.
    pub name: String,
    /// The catalog's functional family (`math`, `text`, ...).
    pub category: String,
    /// Excel-evidence cells the run found (0 when none).
    pub cells: usize,
}

/// The catalog functions below the floor, by category, then fewest cells, then name. Functions
/// the oracle cannot evaluate (`oracle: none`) are listed elsewhere in the report and left out.
#[must_use]
pub fn below_floor(
    catalog: &Catalog,
    evidence: &BTreeMap<String, FunctionEvidence>,
) -> Vec<BelowFloor> {
    let mut below: Vec<BelowFloor> = catalog
        .functions()
        .filter(|f| catalog.in_scope(&f.name) && !catalog.needs_other_evidence(&f.name))
        .map(|f| BelowFloor {
            name: f.name.clone(),
            category: f.category.clone(),
            cells: evidence.get(&f.name).map_or(0, |e| e.cells),
        })
        .filter(|f| f.cells < EVIDENCE_FLOOR)
        .collect();
    below.sort_by(|a, b| (&a.category, a.cells, &a.name).cmp(&(&b.category, b.cells, &b.name)));
    below
}

/// The report section: one line per category, `NAME (cells)` in order.
pub fn write_markdown(md: &mut String, below: &[BelowFloor]) {
    let _ = writeln!(
        md,
        "\n### Below the evidence floor ({} functions with fewer than {EVIDENCE_FLOOR} Excel cells)\n",
        below.len()
    );
    let mut by_category: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for f in below {
        by_category
            .entry(&f.category)
            .or_default()
            .push(format!("{} ({})", f.name, f.cells));
    }
    for (category, names) in by_category {
        let _ = writeln!(md, "- {category} ({}): {}", names.len(), names.join(", "));
    }
}

/// The committed ratchet.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    /// The floor the count was taken at; a lock taken at another floor is refused.
    pub floor: usize,
    /// In-scope functions below the floor.
    pub below_floor: usize,
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
    /// Reads [`LOCK_PATH`] under `root`. A missing lock allows no function below the floor, so
    /// the first one is written with `--update --allow-increase`.
    ///
    /// # Errors
    /// When the file cannot be read or is malformed.
    pub fn read(root: &Path) -> Result<Lock, LockError> {
        let path = root.join(LOCK_PATH);
        match std::fs::read_to_string(&path) {
            Ok(text) => Ok(serde_json::from_str(&text)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Lock {
                floor: EVIDENCE_FLOOR,
                below_floor: 0,
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

/// What the ratchet decided for a count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The count equals the lock.
    Unchanged,
    /// Fewer functions are below the floor: record it (`--update`).
    Improved,
    /// More functions are below the floor than the lock allows.
    Regressed,
    /// The lock was taken at another floor.
    FloorChanged,
}

/// Compares `count` (functions below [`EVIDENCE_FLOOR`]) with `lock`.
#[must_use]
pub fn judge(lock: Lock, count: usize) -> Verdict {
    if lock.floor != EVIDENCE_FLOOR {
        Verdict::FloorChanged
    } else if count > lock.below_floor {
        Verdict::Regressed
    } else if count < lock.below_floor {
        Verdict::Improved
    } else {
        Verdict::Unchanged
    }
}

/// Whether `--update` may write the lock for `verdict`: a drop always, a rise or a lock taken at
/// another floor only with `allow_increase` (decision 0062).
#[must_use]
pub fn may_record(verdict: Verdict, allow_increase: bool) -> bool {
    match verdict {
        Verdict::Unchanged | Verdict::Improved => true,
        Verdict::Regressed | Verdict::FloorChanged => allow_increase,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogFunction, OracleSupport, Scope};

    fn entry(
        name: &str,
        category: &str,
        scope: Scope,
        oracle: Option<OracleSupport>,
    ) -> CatalogFunction {
        CatalogFunction {
            name: name.into(),
            category: category.into(),
            milestone: "M5".into(),
            scope,
            reason: None,
            oracle,
        }
    }

    fn evidence(name: &str, cells: usize) -> (String, FunctionEvidence) {
        (
            name.to_string(),
            FunctionEvidence {
                name: name.to_string(),
                cells,
                ..FunctionEvidence::default()
            },
        )
    }

    #[test]
    fn lists_thin_and_missing_in_scope_functions_by_category() {
        let catalog = Catalog::from_functions([
            entry("ABS", "math", Scope::In, None),
            entry("COS", "math", Scope::In, None),
            entry("SUM", "math", Scope::In, None),
            entry("LEN", "text", Scope::In, None),
            entry(
                "INFO",
                "information",
                Scope::In,
                Some(OracleSupport::Unavailable),
            ),
            entry("RTD", "lookup", Scope::Out, None),
        ]);
        let ev: BTreeMap<String, FunctionEvidence> = [
            evidence("ABS", 19),
            evidence("SUM", EVIDENCE_FLOOR),
            evidence("LEN", 3),
        ]
        .into_iter()
        .collect();
        let below = below_floor(&catalog, &ev);
        let names: Vec<(&str, &str, usize)> = below
            .iter()
            .map(|f| (f.category.as_str(), f.name.as_str(), f.cells))
            .collect();
        assert_eq!(
            names,
            [("math", "COS", 0), ("math", "ABS", 19), ("text", "LEN", 3)]
        );
        let mut md = String::new();
        write_markdown(&mut md, &below);
        assert!(md.contains("(3 functions with fewer than 20 Excel cells)"));
        assert!(md.contains("- math (2): COS (0), ABS (19)\n- text (1): LEN (3)"));
    }

    #[test]
    fn ratchet_only_goes_down_at_the_same_floor() {
        let lock = Lock {
            floor: EVIDENCE_FLOOR,
            below_floor: 10,
        };
        assert_eq!(judge(lock, 10), Verdict::Unchanged);
        assert_eq!(judge(lock, 9), Verdict::Improved);
        assert_eq!(judge(lock, 11), Verdict::Regressed);
        let other = Lock {
            floor: EVIDENCE_FLOOR + 5,
            below_floor: 10,
        };
        assert_eq!(judge(other, 10), Verdict::FloorChanged);
    }

    #[test]
    fn update_records_a_rise_or_a_new_floor_only_when_allowed() {
        for verdict in [Verdict::Unchanged, Verdict::Improved] {
            assert!(may_record(verdict, false));
        }
        for verdict in [Verdict::Regressed, Verdict::FloorChanged] {
            assert!(!may_record(verdict, false));
            assert!(may_record(verdict, true));
        }
    }

    #[test]
    fn lock_rejects_unknown_fields() {
        assert!(serde_json::from_str::<Lock>(r#"{"floor": 20, "below_floor": 3}"#).is_ok());
        assert!(
            serde_json::from_str::<Lock>(r#"{"floor": 20, "below_floor": 3, "x": 1}"#).is_err()
        );
    }
}
