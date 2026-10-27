//! The evidence floor (T-0056): every in-scope catalog function the oracle can evaluate should
//! have at least [`EVIDENCE_FLOOR`] cells with Excel-sourced expectations. The report lists the
//! functions below it by category, and `quality/evidence-floor.json` ratchets their number down
//! (decision 0062): `rustsheet-eval evidence-floor` fails when it rises or when the lock is stale,
//! and `--update` records a drop. `rustsheet-eval check --evidence-floor` judges the floor from
//! the same run as the baseline gate ([`plan_run`], [`floor_results`]; T-2761).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::lock_file;
pub use crate::lock_file::LockError;
use crate::report::FunctionEvidence;
use crate::runner::CaseResult;
use crate::suite::Suite;

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

impl Lock {
    /// Reads [`LOCK_PATH`] under `root`. A missing lock allows no function below the floor, so
    /// the first one is written with `--update --allow-increase`.
    ///
    /// # Errors
    /// When the file cannot be read or is malformed.
    pub fn read(root: &Path) -> Result<Lock, LockError> {
        lock_file::read(
            root,
            LOCK_PATH,
            Lock {
                floor: EVIDENCE_FLOOR,
                below_floor: 0,
            },
        )
    }

    /// Writes the lock to [`LOCK_PATH`] under `root`.
    ///
    /// # Errors
    /// When the file cannot be written.
    pub fn write(&self, root: &Path) -> Result<(), LockError> {
        lock_file::write(root, LOCK_PATH, self)
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

/// Whether `suite` counts toward the floor: vendored suites only, so fetching an external suite
/// cannot move the count (decision 0062).
#[must_use]
pub fn counts(suite: &Suite) -> bool {
    suite.manifest.fetch.is_none()
}

/// The suites one run needs to judge both the baseline gate on `gate` and the floor: `gate`
/// unchanged and first, then each suite of `all` that [`counts`] and `gate` does not hold. The
/// first `gate.len()` suites of the plan are exactly `gate` (T-2761).
#[must_use]
pub fn plan_run(gate: Vec<Suite>, all: Vec<Suite>) -> Vec<Suite> {
    let mut plan = gate;
    let extra: Vec<Suite> = all
        .into_iter()
        .filter(|s| counts(s) && !plan.iter().any(|g| g.manifest.name == s.manifest.name))
        .collect();
    plan.extend(extra);
    plan
}

/// The results of a run of `suites` that the floor counts: those of the suites that [`counts`],
/// the same set `rustsheet-eval evidence-floor` runs on its own.
#[must_use]
pub fn floor_results(suites: &[Suite], results: &[CaseResult]) -> Vec<CaseResult> {
    let counted: Vec<&str> = suites
        .iter()
        .filter(|s| counts(s))
        .map(|s| s.manifest.name.as_str())
        .collect();
    results
        .iter()
        .filter(|r| counted.contains(&r.suite.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogFunction, OracleSupport, Scope};
    use crate::runner::CaseStatus;
    use crate::suite::SuiteManifest;
    use std::path::PathBuf;

    /// A suite named `name`; `fetched` gives it a `[fetch]` table (an external suite).
    fn suite(name: &str, gate: bool, fetched: bool) -> Suite {
        let fetch = if fetched {
            "[fetch]\nurl = \"https://example.com/r.git\"\nrev = \"0123456789abcdef0123456789abcdef01234567\"\n"
        } else {
            ""
        };
        let text = format!(
            "name = \"{name}\"\nkind = \"spec\"\nlicense = \"MIT\"\ngate = {gate}\n{fetch}"
        );
        let manifest: SuiteManifest = toml::from_str(&text).expect("manifest parses");
        Suite {
            manifest,
            dir: PathBuf::from("eval/suites").join(name),
            cache_dir: PathBuf::from("eval/.cache/suites").join(name),
        }
    }

    fn names(suites: &[Suite]) -> Vec<&str> {
        suites.iter().map(|s| s.manifest.name.as_str()).collect()
    }

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
            v15: None,
            v15_reason: None,
            params: Vec::new(),
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

    #[test]
    fn plan_keeps_the_gate_suites_first_and_adds_the_other_vendored_ones() {
        // gate-v: vendored gate suite; gate-f: fetched gate suite; extra-v: vendored, not gated;
        // extra-f: fetched, not gated (never run for the floor).
        let all = || {
            vec![
                suite("extra-f", false, true),
                suite("extra-v", false, false),
                suite("gate-f", true, true),
                suite("gate-v", true, false),
            ]
        };
        let gate: Vec<Suite> = all().into_iter().filter(|s| s.manifest.gate).collect();
        let plan = plan_run(gate, all());
        assert_eq!(names(&plan), ["gate-f", "gate-v", "extra-v"]);
        // The standalone command runs exactly the suites that count; the plan holds all of them.
        let standalone: Vec<Suite> = all().into_iter().filter(counts).collect();
        assert_eq!(names(&standalone), ["extra-v", "gate-v"]);
        assert!(standalone
            .iter()
            .all(|s| names(&plan).contains(&s.manifest.name.as_str())));
    }

    #[test]
    fn floor_counts_only_the_vendored_suites_results() {
        let plan = vec![
            suite("gate-f", true, true),
            suite("gate-v", true, false),
            suite("extra-v", false, false),
        ];
        let results: Vec<CaseResult> = ["gate-f", "gate-v", "extra-v", "gate-v"]
            .iter()
            .enumerate()
            .map(|(i, s)| CaseResult::empty(&format!("{s}/case{i}"), s, CaseStatus::Pass))
            .collect();
        let ids: Vec<String> = floor_results(&plan, &results)
            .into_iter()
            .map(|r| r.id)
            .collect();
        assert_eq!(ids, ["gate-v/case1", "extra-v/case2", "gate-v/case3"]);
    }
}
