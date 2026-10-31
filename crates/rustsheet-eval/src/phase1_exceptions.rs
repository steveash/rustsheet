//! Cells left out of the Phase 1 rate (decision 0134): visibly-off Excel cells whose only open
//! question is desktop Excel evidence, which decision 0039 assigns to M14 and says never block a
//! phase exit. One line per cell, tab-separated: `case!Sheet!A1`, the decision that names the
//! cell as M14-only, and the open M14 task that owns it.
//!
//! The list only changes how `v15_checkpoint` counts a function's Phase 1 cells: it hides no cell
//! from the baseline or the gate, and every listed cell stays a known failure. The exit
//! exceptions of decision 0087 ([`crate::exit_exceptions`]) are a separate list for a separate
//! measure.
//!
//! The rate also leaves out the cells of [`DESKTOP_OWNED_PATH`] (decision 0143): known failures
//! that only open M14 tasks own, which `scripts/desktop_owned.py` lists from the baselines and the
//! task files before each `make eval` (`rustsheet-eval run --desktop-owned`). That list is
//! generated, never committed, and needs no decision per cell.

use std::collections::BTreeMap;
use std::path::Path;

use crate::exit_exceptions::{check_owner, check_task, read_optional, split_cell, ExceptionsError};
use crate::probe_milestones::data_lines;

/// The list, relative to the repository root.
pub const PHASE1_EXCEPTIONS_PATH: &str = "eval/milestones/phase1-exceptions.tsv";

/// The generated list of cells only open M14 tasks own (decision 0143), relative to the
/// repository root: `case!Sheet!A1<TAB>T-<owner>` per line, written by `scripts/desktop_owned.py`.
pub const DESKTOP_OWNED_PATH: &str = "eval/runs/desktop-owned.tsv";

/// One line of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase1Exception {
    /// 1-based line number.
    pub line: usize,
    /// The decision that names the cell, four digits.
    pub decision: String,
    /// The M14 task that owns the cell, `T-<digits>`.
    pub task: String,
}

/// The parsed list, keyed by case id and `Sheet!A1` ([`crate::runner::CellResult::key`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Phase1Exceptions {
    /// `(case, Sheet!A1)` → its line.
    pub cells: BTreeMap<(String, String), Phase1Exception>,
    /// `(case, Sheet!A1)` → the M14 task that alone owns it ([`DESKTOP_OWNED_PATH`]).
    pub desktop_owned: BTreeMap<(String, String), String>,
}

impl Phase1Exceptions {
    /// Parses the list's text: `case!Sheet!A1<TAB>decision<TAB>task` per line; `#` starts a
    /// comment line.
    ///
    /// # Errors
    /// The first malformed line, or the first that repeats a cell.
    pub fn parse(text: &str) -> Result<Phase1Exceptions, ExceptionsError> {
        let mut exceptions = Phase1Exceptions::default();
        for (line, fields) in data_lines(text) {
            let bad = |why| ExceptionsError::Line {
                path: PHASE1_EXCEPTIONS_PATH,
                line,
                why,
            };
            let [cell, decision, task] = fields.as_slice() else {
                return Err(bad("expected `case!Sheet!A1<TAB>decision<TAB>task`"));
            };
            let cell = split_cell(cell).ok_or(bad("the cell must be `suite/case!Sheet!A1`"))?;
            check_owner(decision, task).map_err(bad)?;
            if let Some(first) = exceptions.cells.get(&cell) {
                return Err(ExceptionsError::Duplicate {
                    path: PHASE1_EXCEPTIONS_PATH,
                    line,
                    cell: crate::runner::cell_id(&cell.0, &cell.1),
                    first: first.line,
                });
            }
            exceptions.cells.insert(
                cell,
                Phase1Exception {
                    line,
                    decision: (*decision).to_string(),
                    task: (*task).to_string(),
                },
            );
        }
        Ok(exceptions)
    }

    /// Reads [`PHASE1_EXCEPTIONS_PATH`] under `repo_root`; an absent file lists no cell.
    ///
    /// # Errors
    /// When the file is unreadable or malformed.
    pub fn load(repo_root: &Path) -> Result<Phase1Exceptions, ExceptionsError> {
        read_optional(repo_root, PHASE1_EXCEPTIONS_PATH)?.map_or_else(
            || Ok(Phase1Exceptions::default()),
            |text| Self::parse(&text),
        )
    }

    /// Adds the cells of the generated desktop-owned list's text: `case!Sheet!A1<TAB>T-<owner>`
    /// per line.
    ///
    /// # Errors
    /// The first malformed line, or the first that repeats a cell.
    pub fn with_desktop_owned(mut self, text: &str) -> Result<Phase1Exceptions, ExceptionsError> {
        for (line, fields) in data_lines(text) {
            let bad = |why| ExceptionsError::Line {
                path: DESKTOP_OWNED_PATH,
                line,
                why,
            };
            let [cell, task] = fields.as_slice() else {
                return Err(bad("expected `case!Sheet!A1<TAB>task`"));
            };
            let cell = split_cell(cell).ok_or(bad("the cell must be `suite/case!Sheet!A1`"))?;
            check_task(task).map_err(bad)?;
            if self.desktop_owned.contains_key(&cell) {
                return Err(bad("the cell is listed twice"));
            }
            self.desktop_owned.insert(cell, (*task).to_string());
        }
        Ok(self)
    }

    /// [`Self::with_desktop_owned`] over [`DESKTOP_OWNED_PATH`] under `repo_root`.
    ///
    /// # Errors
    /// When the file is absent (`make eval` writes it), unreadable or malformed.
    pub fn load_desktop_owned(self, repo_root: &Path) -> Result<Phase1Exceptions, ExceptionsError> {
        let text = read_optional(repo_root, DESKTOP_OWNED_PATH)?.ok_or_else(|| {
            ExceptionsError::Unreadable {
                path: DESKTOP_OWNED_PATH,
                message: "not found; `scripts/desktop_owned.py --out` writes it (make eval does)"
                    .to_string(),
            }
        })?;
        self.with_desktop_owned(&text)
    }

    /// Whether the cell `key` (`Sheet!A1`) of `case` is listed in [`PHASE1_EXCEPTIONS_PATH`].
    #[must_use]
    pub fn contains(&self, case: &str, key: &str) -> bool {
        self.cells
            .contains_key(&(case.to_string(), key.to_string()))
    }

    /// Whether the cell `key` (`Sheet!A1`) of `case` is in the desktop-owned list.
    #[must_use]
    pub fn desktop_owns(&self, case: &str, key: &str) -> bool {
        self.desktop_owned
            .contains_key(&(case.to_string(), key.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const J136: &str = "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!J136";

    #[test]
    fn parses_lines_and_skips_comments() {
        let text = format!("# comment\n\n{J136}\t0134\tT-1075\n");
        let exceptions = Phase1Exceptions::parse(&text).expect("parses");
        assert!(exceptions.contains("poi/FormulaEvalTestData_Copy.xlsx", "EverythingTests!J136"));
        assert!(!exceptions.contains("poi/FormulaEvalTestData_Copy.xlsx", "EverythingTests!J137"));
        let quoted = "poi/a.xlsx!'My Sheet'!$b$2\t0134\tT-1075";
        let exceptions = Phase1Exceptions::parse(quoted).expect("parses");
        assert!(exceptions.contains("poi/a.xlsx", "My Sheet!B2"));
    }

    #[test]
    fn rejects_a_line_without_a_decision_or_an_owner() {
        for bad in [
            format!("{J136}\t0134"),
            format!("{J136}\tT-1075"),
            format!("{J136}\t0134\tT-1075\textra"),
            format!("{J136}\t134\tT-1075"),
            format!("{J136}\t0134\t1075"),
            format!("{J136}\t0134\tT-"),
            "poi/a.xlsx!Sheet1\t0134\tT-1075".to_string(),
            "poi/a.xlsx!Sheet1!nope\t0134\tT-1075".to_string(),
        ] {
            assert!(
                matches!(
                    Phase1Exceptions::parse(&bad),
                    Err(ExceptionsError::Line { line: 1, .. })
                ),
                "{bad}"
            );
        }
        let twice = format!("{J136}\t0134\tT-1075\n{J136}\t0067\tT-1075\n");
        assert!(matches!(
            Phase1Exceptions::parse(&twice),
            Err(ExceptionsError::Duplicate {
                line: 2,
                first: 1,
                ..
            })
        ));
    }

    #[test]
    fn reads_the_desktop_owned_list() {
        let text = "gen-m9/a.case.json!Calc!A4\tT-2425\npoi/a.xlsx!'My Sheet'!$b$2\tT-1075\n";
        let exceptions = Phase1Exceptions::default()
            .with_desktop_owned(text)
            .expect("parses");
        assert!(exceptions.desktop_owns("gen-m9/a.case.json", "Calc!A4"));
        assert!(exceptions.desktop_owns("poi/a.xlsx", "My Sheet!B2"));
        assert!(!exceptions.contains("gen-m9/a.case.json", "Calc!A4"));
        for bad in [
            "gen-m9/a.case.json!Calc!A4".to_string(),
            "gen-m9/a.case.json!Calc!A4\t0143\tT-2425".to_string(),
            "gen-m9/a.case.json!Calc!A4\t2425".to_string(),
            "gen-m9/a.case.json!Calc\tT-2425".to_string(),
            "gen-m9/a.case.json!Calc!A4\tT-2425\ngen-m9/a.case.json!Calc!A4\tT-1553".to_string(),
        ] {
            assert!(
                matches!(
                    Phase1Exceptions::default().with_desktop_owned(&bad),
                    Err(ExceptionsError::Line { .. })
                ),
                "{bad}"
            );
        }
        let missing = std::env::temp_dir().join(format!("no-desktop-owned-{}", std::process::id()));
        assert!(matches!(
            Phase1Exceptions::default().load_desktop_owned(&missing),
            Err(ExceptionsError::Unreadable { .. })
        ));
    }

    #[test]
    fn the_committed_file_parses() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let exceptions = Phase1Exceptions::load(&root).unwrap_or_else(|e| panic!("{e}"));
        let cells: Vec<String> = exceptions
            .cells
            .iter()
            .map(|((case, key), e)| format!("{case}!{key} {} {}", e.decision, e.task))
            .collect();
        assert_eq!(
            cells,
            [
                "ironcalc-calc/ENGINEERING/COMPLEXs.xlsx!Sheet1!AL12 0188 T-1075",
                "ironcalc-calc/ENGINEERING/COMPLEXs.xlsx!Sheet1!AL26 0188 T-1075",
                "ironcalc-calc/ENGINEERING/COMPLEXs.xlsx!Sheet1!K25 0196 T-1075",
                "ironcalc-calc/ENGINEERING/COMPLEXs.xlsx!Sheet1!N6 0188 T-1075",
                "ironcalc-calc/FINANCIAL/IRR.xlsx!Two Values!C4 0196 T-1075",
                "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!G936 0134 T-1075",
                "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!H936 0134 T-1075",
                "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!J136 0134 T-1075",
                "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!K936 0134 T-1075",
            ]
        );
    }
}
