//! Cells left out of the Phase 1 rate (decision 0134): visibly-off Excel cells whose only open
//! question is desktop Excel evidence, which decision 0039 assigns to M14 and says never block a
//! phase exit. One line per cell, tab-separated: `case!Sheet!A1`, the decision that names the
//! cell as M14-only, and the open M14 task that owns it.
//!
//! The list only changes how `v15_checkpoint` counts a function's Phase 1 cells: it hides no cell
//! from the baseline or the gate, and every listed cell stays a known failure. The exit
//! exceptions of decision 0087 ([`crate::exit_exceptions`]) are a separate list for a separate
//! measure.

use std::collections::BTreeMap;
use std::path::Path;

use crate::exit_exceptions::{check_owner, read_optional, split_cell, ExceptionsError};
use crate::probe_milestones::data_lines;

/// The list, relative to the repository root.
pub const PHASE1_EXCEPTIONS_PATH: &str = "eval/milestones/phase1-exceptions.tsv";

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
                    cell: format!("{}!{}", cell.0, cell.1),
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

    /// Whether the cell `key` (`Sheet!A1`) of `case` is listed.
    #[must_use]
    pub fn contains(&self, case: &str, key: &str) -> bool {
        self.cells
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
                "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!G936 0134 T-1075",
                "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!H936 0134 T-1075",
                "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!J136 0134 T-1075",
                "poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!K936 0134 T-1075",
            ]
        );
    }
}
