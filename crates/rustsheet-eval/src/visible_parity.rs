//! The visible-parity ratchet (T-0061, decision 0088): the in-scope cells of the vendored gated
//! suites that are visibly off from Excel (decision 0083) may not grow. `rustsheet-eval check`
//! holds a full gated run to [`LOCK_PATH`] with [`judge`]; `update-baseline` records a rise that
//! new cases account for, and only `update-baseline --lower-visible-lock` (the steward, daily)
//! records a drop. A PR that only fixes cells leaves the lock alone, so parallel fix PRs do not
//! all conflict on its one number (T-2245, decision 0093).

use std::fmt::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::closeness::{self, Visible};
use crate::formula_calls::{calls_only_in_scope, functions_in};
use crate::lock_file;
pub use crate::lock_file::LockError;
use crate::report::{FunctionStat, RunReport};
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

impl Lock {
    /// Reads [`LOCK_PATH`] under `root`. A missing lock allows no visibly-off cell.
    ///
    /// # Errors
    /// When the file cannot be read or is malformed.
    pub fn read(root: &Path) -> Result<Lock, LockError> {
        lock_file::read(
            root,
            LOCK_PATH,
            Lock {
                visibly_off_cells: 0,
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
            .filter(|c| Visible::is_off(c))
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

/// How `update-baseline` may move the lock.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Update {
    /// `--allow-regressions`: record any rise (needs a decision record).
    pub allow_regressions: bool,
    /// `--lower-visible-lock`: record a drop (the steward's daily run, decision 0093).
    pub lower: bool,
}

/// What the gate decided about a run's count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judgement {
    /// The count is at or below the lock, and nothing is to be recorded. Below the lock is slack
    /// that the steward's daily `--lower-visible-lock` run takes back (decision 0093).
    Holds,
    /// Record the count (`update-baseline` only).
    Record,
    /// More cells are visibly off than the lock (plus, when recording, the new coverage) allows.
    Regressed,
}

/// Judges `count` against `lock`. `update` is `None` for `check`, which fails only a rise. For
/// `update-baseline` it records a rise that new cases account for, any rise with
/// `--allow-regressions`, and a drop only with `--lower-visible-lock`.
#[must_use]
pub fn judge(lock: Lock, count: VisibleCount, update: Option<Update>) -> Judgement {
    let locked = lock.visibly_off_cells;
    match update {
        _ if count.cells == locked => Judgement::Holds,
        // A drop is recorded only for the steward's daily run; for everyone else it is slack.
        Some(u) if count.cells < locked && u.lower => Judgement::Record,
        _ if count.cells < locked => Judgement::Holds,
        Some(u)
            if u.allow_regressions || count.cells <= locked.saturating_add(count.new_coverage) =>
        {
            Judgement::Record
        }
        None | Some(_) => Judgement::Regressed,
    }
}

/// The report's visibly-off section: cells per suite, then the functions they are blamed on,
/// worst first (decision 0083).
pub fn write_markdown(report: &RunReport, md: &mut String) {
    let _ = writeln!(md, "\n## Visibly off (fifteen digits)\n");
    let _ = writeln!(
        md,
        "{} in-scope cells are visibly off: a number further than half a unit in the 15th significant digit of Excel's (or 0 against nonzero), or a wrong type, error, text or boolean. `digits15` cells are within that half unit but more than 4 ulp away.\n",
        report.visibly_off_cells()
    );
    let _ = writeln!(
        md,
        "| suite | scored | bit-exact | ≤4 ulp | digits15 | visible numeric | visible other |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|---|");
    for s in &report.suites {
        let c = &s.closeness;
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} | {} |",
            s.name,
            c.scored(),
            c.pass,
            c.ulp1 + c.ulp4,
            c.digits15,
            c.visible_numeric,
            c.visible_other
        );
    }
    let mut off: Vec<&FunctionStat> = report
        .functions
        .iter()
        .filter(|f| f.visible.total() > 0)
        .collect();
    off.sort_by(|a, b| {
        b.visible
            .total()
            .cmp(&a.visible.total())
            .then_with(|| a.name.cmp(&b.name))
    });
    let _ = writeln!(
        md,
        "\nFunctions with visibly-off cells ({}), worst first. A cell is blamed on the functions it calls other than glue ({}), or on its glue when it calls nothing else:\n",
        off.len(),
        closeness::GLUE.join(", ")
    );
    let _ = writeln!(md, "| function | visibly off | numeric | other | checked |");
    let _ = writeln!(md, "|---|---|---|---|---|");
    for f in off {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} |",
            f.name,
            f.visible.total(),
            f.visible.numeric,
            f.visible.other,
            f.cells
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::tests::{case, cell};
    use crate::runner::CellResult;
    use crate::value::{Value, Verdict};

    fn count(cells: usize, new_coverage: usize) -> VisibleCount {
        VisibleCount {
            cells,
            new_coverage,
        }
    }

    const PR: Update = Update {
        allow_regressions: false,
        lower: false,
    };

    #[test]
    fn check_fails_only_a_rise() {
        let lock = Lock {
            visibly_off_cells: 10,
        };
        assert_eq!(judge(lock, count(10, 0), None), Judgement::Holds);
        assert_eq!(judge(lock, count(9, 0), None), Judgement::Holds);
        assert_eq!(judge(lock, count(11, 5), None), Judgement::Regressed);
    }

    /// A PR's update leaves a drop unrecorded, so fix PRs do not conflict on the lock
    /// (decision 0093); the steward's `--lower-visible-lock` records it.
    #[test]
    fn update_records_new_coverage_and_only_the_steward_records_drops() {
        let lock = Lock {
            visibly_off_cells: 10,
        };
        assert_eq!(judge(lock, count(7, 0), Some(PR)), Judgement::Holds);
        let steward = Update { lower: true, ..PR };
        assert_eq!(judge(lock, count(7, 0), Some(steward)), Judgement::Record);
        assert_eq!(judge(lock, count(13, 3), Some(PR)), Judgement::Record);
        assert_eq!(judge(lock, count(13, 2), Some(PR)), Judgement::Regressed);
        assert_eq!(
            judge(lock, count(13, 2), Some(steward)),
            Judgement::Regressed
        );
        let allow = Update {
            allow_regressions: true,
            ..PR
        };
        assert_eq!(judge(lock, count(13, 0), Some(allow)), Judgement::Record);
        assert_eq!(judge(lock, count(10, 4), Some(PR)), Judgement::Holds);
        // New cases inside the slack below the lock need no write either.
        assert_eq!(judge(lock, count(9, 4), Some(PR)), Judgement::Holds);
    }

    #[test]
    fn lock_rejects_unknown_fields_and_the_committed_lock_parses() {
        assert!(serde_json::from_str::<Lock>(r#"{"visibly_off_cells": 1, "x": 2}"#).is_err());
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(Lock::read(&root).is_ok());
    }

    /// Visibly-off cells are blamed on the specialised function, not on the glue around it, and
    /// each suite counts its in-scope visibly-off cells for the ratchet (T-0061).
    #[test]
    fn visibly_off_cells_are_blamed_past_glue_and_counted_per_suite() {
        let mut r = case("probes/a", 1);
        let off = |formula: &str| CellResult {
            actual: Value::Number(2.0),
            ..cell(formula, Verdict::NumericMismatch)
        };
        r.cells = vec![
            off("IF(TRUE,GAMMA(10))"),
            off("SUM(1,2)"),
            cell("WEBSERVICE(\"x\")", Verdict::TypeMismatch),
            cell("GAMMA(2)", Verdict::Pass),
        ];
        let catalog = Catalog::from_functions([crate::catalog::CatalogFunction {
            name: "WEBSERVICE".into(),
            category: "web".into(),
            milestone: "M5".into(),
            scope: crate::catalog::Scope::Out,
            reason: None,
            oracle: None,
            v15: None,
            v15_reason: None,
            params: Vec::new(),
        }]);
        let report = RunReport::build("e", &[r], &[], &catalog);
        let visible = |name: &str| {
            report
                .functions
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.visible.total())
        };
        assert_eq!(visible("GAMMA"), Some(1));
        assert_eq!(visible("IF"), Some(0));
        assert_eq!(visible("SUM"), Some(1));
        assert_eq!(report.suites[0].closeness.scored(), 3);
        assert_eq!(report.visibly_off_cells(), 2);
        let md = report.to_markdown(1);
        assert!(md.contains("2 in-scope cells are visibly off"), "{md}");
        assert!(md.contains("| GAMMA | 1 | 1 | 0 | 2 |"), "{md}");
    }
}
