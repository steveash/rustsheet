//! Named milestone exit exceptions (decision 0087): the `ironcalc-calc` cells that a decision
//! leaves out of a milestone's function-scope exit measure (docs/milestones.md, M2 and M5; decisions
//! 0074, 0085 and 0086). One line per cell, tab-separated: `case!Sheet!A1`, the milestone from which the
//! exception applies, the decision that names it, and the task that owns the cell.
//!
//! The file only changes how the "Milestone function scope" rows are read: it hides no cell from
//! the baseline or the gate, and every listed cell stays a known failure until it is fixed.

use std::collections::BTreeMap;
use std::path::Path;

use crate::case::split_qualified;
use crate::cellref::Addr;
use crate::function_scope::SUITE;
use crate::probe_milestones::{data_lines, parse_milestone};

/// The exceptions file, relative to the repository root.
pub const EXCEPTIONS_PATH: &str = "eval/milestones/exit-exceptions.tsv";

/// One line of the exceptions file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exception {
    /// 1-based line number.
    pub line: usize,
    /// The first milestone whose row leaves the cell out (`M5` → 5); later rows do too.
    pub milestone: u32,
    /// The decision that names the cell, four digits.
    pub decision: String,
    /// The task that owns the cell, `T-<digits>`.
    pub task: String,
}

/// The parsed exceptions file, keyed by cell as the function-scope rows name failing cells
/// (`case!Sheet!A1`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExitExceptions {
    /// Cell → its exception.
    pub cells: BTreeMap<String, Exception>,
}

/// Why an exceptions file (this one, or the Phase 1 list of
/// [`crate::phase1_exceptions`]) or one of its lines is unusable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ExceptionsError {
    /// The file cannot be read.
    #[error("{path}: {message}")]
    Unreadable { path: &'static str, message: String },
    /// A line is malformed.
    #[error("{path} line {line}: {why}")]
    Line {
        path: &'static str,
        line: usize,
        why: &'static str,
    },
    /// A cell is listed twice.
    #[error("{path} line {line}: {cell} is already listed on line {first}")]
    Duplicate {
        path: &'static str,
        line: usize,
        cell: String,
        first: usize,
    },
}

/// `case!Sheet!A1` split into the case id and `Sheet!A1` with a canonical address, the form
/// [`crate::runner::CellResult::key`] gives; `None` when malformed.
pub(crate) fn split_cell(cell: &str) -> Option<(String, String)> {
    let (case, qualified) = cell.split_once('!')?;
    let (sheet, a1) = split_qualified(qualified)?;
    if case.is_empty() || sheet.is_empty() {
        return None;
    }
    Some((case.to_string(), format!("{sheet}!{}", Addr::parse(&a1)?)))
}

/// `ironcalc-calc/<path>!Sheet!A1` in the form the function-scope rows use, or `None`.
fn cell_id(cell: &str) -> Option<String> {
    let (case, key) = split_cell(cell)?;
    case.starts_with(&format!("{SUITE}/"))
        .then(|| format!("{case}!{key}"))
}

/// Checks a line's decision (four digits) and owning task (`T-<digits>`) fields.
pub(crate) fn check_owner(decision: &str, task: &str) -> Result<(), &'static str> {
    if decision.len() != 4 || !decision.bytes().all(|b| b.is_ascii_digit()) {
        return Err("the decision must be four digits");
    }
    let digits = task.strip_prefix("T-").unwrap_or_default();
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err("the task must be `T-<digits>`");
    }
    Ok(())
}

/// Reads the file at `path` under `repo_root`; `None` when it is absent.
pub(crate) fn read_optional(
    repo_root: &Path,
    path: &'static str,
) -> Result<Option<String>, ExceptionsError> {
    let full = repo_root.join(path);
    if !full.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(&full)
        .map(Some)
        .map_err(|e| ExceptionsError::Unreadable {
            path,
            message: e.to_string(),
        })
}

impl ExitExceptions {
    /// Parses the file's text: `cell<TAB>milestone<TAB>decision<TAB>task` per line; `#` starts a
    /// comment line.
    ///
    /// # Errors
    /// The first malformed line, or the first that repeats a cell.
    pub fn parse(text: &str) -> Result<ExitExceptions, ExceptionsError> {
        let mut exceptions = ExitExceptions::default();
        for (line, fields) in data_lines(text) {
            let bad = |why| ExceptionsError::Line {
                path: EXCEPTIONS_PATH,
                line,
                why,
            };
            let [cell, milestone, decision, task] = fields.as_slice() else {
                return Err(bad(
                    "expected `case!Sheet!A1<TAB>milestone<TAB>decision<TAB>task`",
                ));
            };
            let cell =
                cell_id(cell).ok_or(bad("the cell must be `ironcalc-calc/<case>!Sheet!A1`"))?;
            let milestone =
                parse_milestone(milestone).ok_or(bad("the milestone must be `M<digits>`"))?;
            check_owner(decision, task).map_err(bad)?;
            let exception = Exception {
                line,
                milestone,
                decision: (*decision).to_string(),
                task: (*task).to_string(),
            };
            if let Some(first) = exceptions.cells.get(&cell) {
                return Err(ExceptionsError::Duplicate {
                    path: EXCEPTIONS_PATH,
                    line,
                    cell,
                    first: first.line,
                });
            }
            exceptions.cells.insert(cell, exception);
        }
        Ok(exceptions)
    }

    /// Reads [`EXCEPTIONS_PATH`] under `repo_root`; an absent file holds no exceptions.
    ///
    /// # Errors
    /// When the file is unreadable or malformed.
    pub fn load(repo_root: &Path) -> Result<ExitExceptions, ExceptionsError> {
        read_optional(repo_root, EXCEPTIONS_PATH)?
            .map_or_else(|| Ok(ExitExceptions::default()), |text| Self::parse(&text))
    }

    /// Whether `cell` is left out of the row of milestone `row`: it is listed with a milestone
    /// at most `row`.
    #[must_use]
    pub fn applies(&self, cell: &str, row: u32) -> bool {
        self.cells.get(cell).is_some_and(|e| e.milestone <= row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const C11: &str = "ironcalc-calc/LOGICAL/IF_ARRAY.xlsx!Sheet1!C11";

    #[test]
    fn parses_lines_and_skips_comments() {
        let text = format!("# comment\n\n{C11}\tM2\t0074\tT-1075\n");
        let exceptions = ExitExceptions::parse(&text).expect("parses");
        assert_eq!(
            exceptions.cells.get(C11),
            Some(&Exception {
                line: 3,
                milestone: 2,
                decision: "0074".to_string(),
                task: "T-1075".to_string(),
            })
        );
        assert!(exceptions.applies(C11, 2));
        assert!(exceptions.applies(C11, 5));
        assert!(!exceptions.applies(C11, 1));
        assert!(!exceptions.applies("ironcalc-calc/a!Sheet1!A1", 5));
    }

    #[test]
    fn rejects_each_kind_of_bad_line() {
        for bad in [
            format!("{C11}\tM2\t0074"),
            format!("{C11}\tM2\t0074\tT-1075\textra"),
            "probes/a.case.json!Sheet1!A1\tM2\t0074\tT-1075".to_string(),
            "ironcalc-calc/a.xlsx!Sheet1\tM2\t0074\tT-1075".to_string(),
            format!("{C11}\t2\t0074\tT-1075"),
            format!("{C11}\tMx\t0074\tT-1075"),
            format!("{C11}\tM2\t74\tT-1075"),
            format!("{C11}\tM2\t007a\tT-1075"),
            format!("{C11}\tM2\t0074\t1075"),
        ] {
            assert!(
                matches!(
                    ExitExceptions::parse(&bad),
                    Err(ExceptionsError::Line { line: 1, .. })
                ),
                "{bad}"
            );
        }
        let twice = format!("{C11}\tM2\t0074\tT-1075\n{C11}\tM5\t0085\tT-1075\n");
        assert_eq!(
            ExitExceptions::parse(&twice),
            Err(ExceptionsError::Duplicate {
                path: EXCEPTIONS_PATH,
                line: 2,
                cell: C11.to_string(),
                first: 1,
            })
        );
    }

    #[test]
    fn an_absent_file_holds_no_exceptions() {
        let dir = std::env::temp_dir().join(format!("exit-exceptions-{}", std::process::id()));
        assert_eq!(ExitExceptions::load(&dir), Ok(ExitExceptions::default()));
    }

    #[test]
    fn the_committed_file_parses() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let exceptions = ExitExceptions::load(&root).unwrap_or_else(|e| panic!("{e}"));
        let lines: Vec<(&str, u32, &str)> = exceptions
            .cells
            .iter()
            .map(|(cell, e)| (cell.as_str(), e.milestone, e.decision.as_str()))
            .collect();
        assert_eq!(
            lines,
            [
                ("ironcalc-calc/LOGICAL/IF_ARRAY.xlsx!Sheet1!C11", 2, "0074"),
                ("ironcalc-calc/LOGICAL/IF_ARRAY.xlsx!Sheet1!C4", 5, "0085"),
                ("ironcalc-calc/LOGICAL/IF_ARRAY.xlsx!Sheet1!E4", 5, "0085"),
                (
                    "ironcalc-calc/MATH_AND_TRIGONOMETRY/ATAN2_POWER.xlsx!Sheet1!D14",
                    5,
                    "0086"
                ),
                (
                    "ironcalc-calc/MATH_AND_TRIGONOMETRY/ATAN2_POWER.xlsx!Sheet1!D26",
                    5,
                    "0086"
                ),
            ]
        );
    }
}
