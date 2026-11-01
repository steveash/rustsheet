//! Gate A's two committed lists (SPEC-0029 "The check" steps 4 and 5).
//!
//! `known-failures.tsv`, one line per failing point carried until its owner task lands:
//!
//! ```text
//! <point id>\t<owner task T-NNNN>\t<note>
//! ```
//!
//! `math-over-excel.tsv`, one line per R3 point or eval cell (decision 0188 R3), citing the
//! `Status: MATH-OVER-EXCEL` entry that covers it:
//!
//! ```text
//! <point or eval cell id>\tdocs/excel-semantics/<file>.md#<anchor>
//! ```
//!
//! `exempt.tsv`, one line per in-scope numeric function that needs no table ("Gate A holds"):
//!
//! ```text
//! <FUNCTION>\t<class exact|composite>\t<reason>
//! ```
//!
//! Blank lines and lines starting with `#` are comments. A file that does not exist is empty.

use std::collections::BTreeMap;
use std::path::Path;

use crate::deviations::{check_entry_status, parse_semantics_ref, DeviationError, EntryStatus};

/// The known-failure list, relative to the accuracy manifest's directory.
pub const KNOWN_FAILURES: &str = "known-failures.tsv";

/// The R3 marks, relative to the accuracy manifest's directory.
pub const MATH_OVER_EXCEL: &str = "math-over-excel.tsv";

/// The table exemptions, relative to the accuracy manifest's directory.
pub const EXEMPT: &str = "exempt.tsv";

/// Why an in-scope numeric function needs no reference table (SPEC-0029, "Gate A holds").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExemptClass {
    /// No floating-point kernel: the result is exact (COUNT, ROW, LEN, MATCH, DATE).
    Exact,
    /// A table cannot drive it with scalar literals; the eval suites named in the reason do.
    Composite,
}

/// One `exempt.tsv` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exemption {
    /// The 1-based line number, for messages.
    pub line: usize,
    pub class: ExemptClass,
    pub reason: String,
}

/// Function name → its exemption.
pub type Exemptions = BTreeMap<String, Exemption>;

/// One `known-failures.tsv` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownFailure {
    /// The 1-based line number, for messages.
    pub line: usize,
    /// The open task that owns the point (`T-3347`).
    pub owner: String,
    pub note: String,
}

/// One `math-over-excel.tsv` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct R3Mark {
    /// The 1-based line number, for messages.
    pub line: usize,
    /// The semantics file relative to the repository root.
    pub semantics_file: String,
    pub anchor: String,
}

/// Why a list is unusable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ListError {
    /// The file exists but cannot be read.
    #[error("{path}: {message}")]
    Unreadable { path: String, message: String },
    /// A line breaks the format.
    #[error("{file} line {line}: {why}")]
    Line {
        file: &'static str,
        line: usize,
        why: &'static str,
    },
}

/// Point id → its known-failure line.
pub type KnownFailures = BTreeMap<String, KnownFailure>;

/// Point or eval cell id → its R3 mark.
pub type R3Marks = BTreeMap<String, R3Mark>;

/// The data lines of the list at `path` with their 1-based numbers, split on tabs; none when the
/// file does not exist.
fn data_lines(path: &Path) -> Result<Vec<(usize, Vec<String>)>, ListError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path).map_err(|e| ListError::Unreadable {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    Ok(text
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|(n, line)| (n + 1, line.split('\t').map(str::to_string).collect()))
        .collect())
}

/// Whether `task` reads `T-` and at least four digits.
fn is_task_id(task: &str) -> bool {
    task.strip_prefix("T-")
        .is_some_and(|d| d.len() >= 4 && d.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether `id` is shaped like a cell id, `<case>!<Sheet>!<A1>`.
fn is_cell_id(id: &str) -> bool {
    id.split('!').count() >= 3 && !id.contains(char::is_whitespace)
}

/// Parses `known-failures.tsv` in `dir`.
///
/// # Errors
/// When the file is unreadable, a line is malformed, or a point is listed twice.
pub fn load_known_failures(dir: &Path) -> Result<KnownFailures, ListError> {
    let mut out = KnownFailures::new();
    for (line, fields) in data_lines(&dir.join(KNOWN_FAILURES))? {
        let bad = |why| ListError::Line {
            file: KNOWN_FAILURES,
            line,
            why,
        };
        let [id, owner, note] = fields.as_slice() else {
            return Err(bad(
                "expected 3 tab-separated fields: point id, owner task, note",
            ));
        };
        if !is_cell_id(id) {
            return Err(bad("the point id must read <case>!<Sheet>!<A1>"));
        }
        if !is_task_id(owner) {
            return Err(bad("the owner must be a task id T-NNNN"));
        }
        if note.trim().is_empty() {
            return Err(bad("the note may not be empty"));
        }
        let entry = KnownFailure {
            line,
            owner: owner.clone(),
            note: note.clone(),
        };
        if out.insert(id.clone(), entry).is_some() {
            return Err(bad("the point is listed twice"));
        }
    }
    Ok(out)
}

/// Parses `math-over-excel.tsv` in `dir`.
///
/// # Errors
/// When the file is unreadable, a line is malformed, or an id is listed twice. Whether each cited
/// entry is a `MATH-OVER-EXCEL` entry is [`check_mark`]'s.
pub fn load_r3_marks(dir: &Path) -> Result<R3Marks, ListError> {
    let mut out = R3Marks::new();
    for (line, fields) in data_lines(&dir.join(MATH_OVER_EXCEL))? {
        let bad = |why| ListError::Line {
            file: MATH_OVER_EXCEL,
            line,
            why,
        };
        let [id, semantics] = fields.as_slice() else {
            return Err(bad("expected 2 tab-separated fields: id, semantics entry"));
        };
        if !is_cell_id(id) {
            return Err(bad("the id must read <case>!<Sheet>!<A1>"));
        }
        let (file, anchor) = parse_semantics_ref(semantics)
            .ok_or_else(|| bad("the entry must read docs/excel-semantics/<file>.md#<anchor>"))?;
        let mark = R3Mark {
            line,
            semantics_file: file.to_string(),
            anchor: anchor.to_string(),
        };
        if out.insert(id.clone(), mark).is_some() {
            return Err(bad("the id is listed twice"));
        }
    }
    Ok(out)
}

/// Parses `exempt.tsv` in `dir`.
///
/// # Errors
/// When the file is unreadable, a line is malformed, or a function is listed twice. Whether each
/// function is an in-scope numeric one without a table is the check's (`crate::accuracy`).
pub fn load_exemptions(dir: &Path) -> Result<Exemptions, ListError> {
    let mut out = Exemptions::new();
    for (line, fields) in data_lines(&dir.join(EXEMPT))? {
        let bad = |why| ListError::Line {
            file: EXEMPT,
            line,
            why,
        };
        let [name, class, reason] = fields.as_slice() else {
            return Err(bad(
                "expected 3 tab-separated fields: function, class, reason",
            ));
        };
        // The catalog's names: upper case, digits, `.` and `_` (T.INV.2T, LOG10, ERF.PRECISE).
        let well_formed = name.starts_with(|c: char| c.is_ascii_uppercase())
            && name
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'.' || b == b'_');
        if !well_formed {
            return Err(bad("the function must be an upper-case catalog name"));
        }
        let class = match class.as_str() {
            "exact" => ExemptClass::Exact,
            "composite" => ExemptClass::Composite,
            _ => return Err(bad("the class must be exact or composite")),
        };
        if reason.trim().is_empty() {
            return Err(bad("the reason may not be empty"));
        }
        let entry = Exemption {
            line,
            class,
            reason: reason.clone(),
        };
        if out.insert(name.clone(), entry).is_some() {
            return Err(bad("the function is listed twice"));
        }
    }
    Ok(out)
}

/// Checks that the entry a mark cites exists and has a `Status: MATH-OVER-EXCEL` line, with the
/// anchor check `deviations` lines use.
///
/// # Errors
/// When the entry is missing or is not a MATH-OVER-EXCEL entry.
pub fn check_mark(mark: &R3Mark, repo_root: &Path) -> Result<(), DeviationError> {
    check_entry_status(
        &mark.semantics_file,
        &mark.anchor,
        EntryStatus::MathOverExcel,
        repo_root,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_with(name: &str, file: &str, text: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("rs-accuracy-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join(file), text).expect("write");
        dir
    }

    #[test]
    fn known_failures_parse_and_refuse_bad_lines() {
        let dir = dir_with(
            "kf-ok",
            KNOWN_FAILURES,
            "# comment\n\naccuracy/IRR/s1!Calc!A3\tT-3347\tIRR #NUM! near 0\n",
        );
        let kf = load_known_failures(&dir).expect("parses");
        std::fs::remove_dir_all(&dir).expect("cleanup");
        assert_eq!(kf["accuracy/IRR/s1!Calc!A3"].owner, "T-3347");
        assert_eq!(kf["accuracy/IRR/s1!Calc!A3"].line, 3);
        for (i, bad) in [
            "accuracy/IRR/s1!Calc!A3\tT-3347",
            "accuracy/IRR/s1!Calc!A3\t3347\tnote",
            "accuracy/IRR/s1!Calc!A3\tT-3347\t ",
            "accuracy/IRR/s1\tT-3347\tnote",
            "a!Calc!A1\tT-3347\tx\na!Calc!A1\tT-3348\ty",
        ]
        .iter()
        .enumerate()
        {
            let dir = dir_with(&format!("kf-bad{i}"), KNOWN_FAILURES, bad);
            let got = load_known_failures(&dir);
            std::fs::remove_dir_all(&dir).expect("cleanup");
            assert!(got.is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_missing_list_is_empty() {
        let dir = std::env::temp_dir().join("rs-accuracy-no-such-dir");
        assert!(load_known_failures(&dir).expect("empty").is_empty());
        assert!(load_r3_marks(&dir).expect("empty").is_empty());
        assert!(load_exemptions(&dir).expect("empty").is_empty());
    }

    #[test]
    fn exemptions_parse_and_refuse_bad_lines() {
        let dir = dir_with(
            "ex-ok",
            EXEMPT,
            "# comment\nCOUNT\texact\tcounts cells\nT.INV.2T\tcomposite\tprobes-numeric\n",
        );
        let ex = load_exemptions(&dir).expect("parses");
        std::fs::remove_dir_all(&dir).expect("cleanup");
        assert_eq!(ex["COUNT"].class, ExemptClass::Exact);
        assert_eq!(ex["COUNT"].line, 2);
        assert_eq!(ex["T.INV.2T"].class, ExemptClass::Composite);
        for (i, bad) in [
            "COUNT\texact",
            "COUNT\tcheap\treason",
            "COUNT\texact\t ",
            "count\texact\treason",
            "1COUNT\texact\treason",
            "COUNT\texact\ta\nCOUNT\tcomposite\tb",
        ]
        .iter()
        .enumerate()
        {
            let dir = dir_with(&format!("ex-bad{i}"), EXEMPT, bad);
            let got = load_exemptions(&dir);
            std::fs::remove_dir_all(&dir).expect("cleanup");
            assert!(got.is_err(), "{bad:?}");
        }
    }

    #[test]
    fn r3_marks_parse_and_refuse_bad_lines() {
        let dir = dir_with(
            "r3-ok",
            MATH_OVER_EXCEL,
            "accuracy/T.INV/s1!Calc!A9\tdocs/excel-semantics/statistics.md#tiny-p\n",
        );
        let marks = load_r3_marks(&dir).expect("parses");
        std::fs::remove_dir_all(&dir).expect("cleanup");
        let mark = &marks["accuracy/T.INV/s1!Calc!A9"];
        assert_eq!(mark.semantics_file, "docs/excel-semantics/statistics.md");
        assert_eq!(mark.anchor, "tiny-p");
        for (i, bad) in [
            "accuracy/T.INV/s1!Calc!A9\tstatistics.md#tiny-p",
            "accuracy/T.INV/s1!Calc!A9\tdocs/excel-semantics/statistics.md",
            "accuracy/T.INV/s1!Calc!A9",
            "a!Calc!A1\tdocs/excel-semantics/s.md#x\na!Calc!A1\tdocs/excel-semantics/s.md#y",
        ]
        .iter()
        .enumerate()
        {
            let dir = dir_with(&format!("r3-bad{i}"), MATH_OVER_EXCEL, bad);
            let got = load_r3_marks(&dir);
            std::fs::remove_dir_all(&dir).expect("cleanup");
            assert!(got.is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_mark_must_cite_a_math_over_excel_entry() {
        let root = dir_with(
            "r3-entry",
            "s.md",
            "# S\n\n## Tiny p\n\nStatus: MATH-OVER-EXCEL (decision 0188)\n\n## Other\n\nStatus: VERIFIED\n",
        );
        std::fs::create_dir_all(root.join("docs/excel-semantics")).expect("mkdir");
        std::fs::rename(root.join("s.md"), root.join("docs/excel-semantics/s.md")).expect("mv");
        let mark = |anchor: &str| R3Mark {
            line: 1,
            semantics_file: "docs/excel-semantics/s.md".into(),
            anchor: anchor.into(),
        };
        let ok = check_mark(&mark("tiny-p"), &root);
        let not_r3 = check_mark(&mark("other"), &root);
        let missing = check_mark(&mark("nope"), &root);
        std::fs::remove_dir_all(&root).expect("cleanup");
        assert!(ok.is_ok());
        assert!(not_r3.is_err());
        assert!(missing.is_err());
    }
}
