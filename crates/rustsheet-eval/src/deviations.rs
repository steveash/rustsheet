//! Cells that differ from Excel by design (decision 0036): a suite's `deviations` file names a
//! scored cell, the decision and the `Status: DEVIATION` semantics entry that accept the
//! difference (decision 0014), and RustSheet's pinned by-design value. Such a cell is scored apart
//! from bit-exact: the pinned value is a *deviation*, Excel's value a pass (the line is stale), and
//! anything else a failure.
//!
//! File format, one cell per line, tab-separated; `#` starts a comment line:
//!
//! ```text
//! <case path relative to the suite root>  <Sheet!A1>  <decision NNNN>  <docs/excel-semantics/<file>.md#<anchor>>  <pinned value>
//! ```
//!
//! The pinned value is an error literal (`#ERROR!`), `TRUE`/`FALSE`, a double-quoted text or a
//! number. A line whose cell is `*` pins the whole case instead (decision 0168): its pinned value
//! reads `refused "<message>"`, and it holds when the engine refuses the case with an error
//! containing `<message>`, as a directive-5 bound refuses a formula before any cell is scored.

use std::path::Path;

use crate::cellref::Addr;
use crate::overrides::{is_inner_relative, parse_cell};
use crate::suite::Suite;
use crate::value::{compare, Value};

/// Directory the semantics reference must point into.
const SEMANTICS_DIR: &str = "docs/excel-semantics/";

/// One `deviations` line.
#[derive(Debug, Clone, PartialEq)]
pub struct Deviation {
    /// Case path relative to the suite root (`cases/lifted-range-rows.case.json`).
    pub case: String,
    /// What the line pins.
    pub pin: Pin,
    /// The decision record that accepts the deviation (`0014`).
    pub decision: String,
    /// The semantics file relative to the repository root (`docs/excel-semantics/arrays.md`).
    pub semantics_file: String,
    /// The heading anchor of the entry within it.
    pub anchor: String,
}

/// What a `deviations` line pins.
#[derive(Debug, Clone, PartialEq)]
pub enum Pin {
    /// One scored cell and RustSheet's by-design value for it.
    Cell {
        sheet: String,
        addr: Addr,
        value: Value,
    },
    /// The whole case, which the engine refuses with an error containing this message
    /// (decision 0168).
    Refusal(String),
}

/// Why a `deviations` file or one of its lines is unusable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeviationError {
    /// A line does not follow the format.
    #[error("deviations line {line}: {why}")]
    Line { line: usize, why: &'static str },
    /// The file, or the semantics file a line cites, cannot be read.
    #[error("{path}: {message}")]
    Unreadable { path: String, message: String },
    /// The cited semantics entry is missing or is not an accepted deviation.
    #[error("{file}#{anchor}: {why}")]
    Entry {
        file: String,
        anchor: String,
        why: &'static str,
    },
    /// A line names a case file the suite does not have.
    #[error("deviations: no case {0}")]
    MissingCase(String),
}

/// How a listed cell scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The engine gives the pinned by-design value.
    Deviation,
    /// The engine matches Excel: the line is stale and should go.
    StalePass,
    /// Neither: the by-design value changed.
    Failure,
}

/// The cell field of a line that pins the whole case.
const WHOLE_CASE: &str = "*";

/// The message of a `refused "<message>"` pinned value; never empty, so it cannot match every
/// error.
fn parse_refusal(text: &str) -> Option<&str> {
    text.strip_prefix("refused \"")
        .and_then(|t| t.strip_suffix('"'))
        .filter(|message| !message.is_empty())
}

/// Parses a pinned value literal.
fn parse_value(text: &str) -> Option<Value> {
    if text.starts_with('#') && text.len() > 1 {
        return Some(Value::Error(text.to_string()));
    }
    match text {
        "TRUE" => return Some(Value::Bool(true)),
        "FALSE" => return Some(Value::Bool(false)),
        _ => {}
    }
    if let Some(inner) = text.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
        return Some(Value::Text(inner.to_string()));
    }
    text.parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
        .map(Value::Number)
}

/// Parses a `deviations` file.
///
/// # Errors
/// Names the first malformed line.
pub fn parse(text: &str) -> Result<Vec<Deviation>, DeviationError> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let bad = |why: &'static str| DeviationError::Line { line: n + 1, why };
        let fields: Vec<&str> = line.split('\t').collect();
        let [case, cell, decision, semantics, pinned] = fields.as_slice() else {
            return Err(bad("expected 5 tab-separated fields"));
        };
        let pin = if *cell == WHOLE_CASE {
            let message = parse_refusal(pinned)
                .ok_or_else(|| bad("a whole case pins `refused \"<message>\"`"))?;
            Pin::Refusal(message.to_string())
        } else {
            let (sheet, addr) = parse_cell(cell).ok_or_else(|| bad("bad cell"))?;
            let value = parse_value(pinned).ok_or_else(|| bad("bad pinned value"))?;
            Pin::Cell { sheet, addr, value }
        };
        if !is_inner_relative(case) {
            return Err(bad("case path must be relative, without `..`"));
        }
        let decision = decision
            .strip_prefix("decision ")
            .filter(|d| d.len() == 4 && d.bytes().all(|b| b.is_ascii_digit()))
            .ok_or_else(|| bad("decision must read `decision NNNN`"))?;
        let (file, anchor) = semantics
            .split_once('#')
            .filter(|(file, anchor)| {
                file.starts_with(SEMANTICS_DIR)
                    && Path::new(file)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                    && is_inner_relative(file)
                    && !anchor.is_empty()
            })
            .ok_or_else(|| bad("semantics must read `docs/excel-semantics/<file>.md#<anchor>`"))?;
        out.push(Deviation {
            case: (*case).to_string(),
            pin,
            decision: decision.to_string(),
            semantics_file: file.to_string(),
            anchor: anchor.to_string(),
        });
    }
    Ok(out)
}

/// The GitHub anchor of a Markdown heading: lower case, spaces to `-`, other punctuation dropped.
fn anchor_of(heading: &str) -> String {
    heading
        .trim()
        .chars()
        .filter_map(|c| match c {
            ' ' => Some('-'),
            '-' | '_' => Some(c),
            c if c.is_alphanumeric() => Some(c.to_ascii_lowercase()),
            _ => None,
        })
        .collect()
}

/// Checks that the cited entry exists and is an accepted deviation: its section (from its
/// heading to the next heading of any level) has a `Status: DEVIATION` line.
///
/// # Errors
/// When the file is unreadable, the anchor names no heading, or the entry is not a DEVIATION.
pub fn check_entry(d: &Deviation, repo_root: &Path) -> Result<(), DeviationError> {
    let text = read(&repo_root.join(&d.semantics_file))?;
    let entry = |why| DeviationError::Entry {
        file: d.semantics_file.clone(),
        anchor: d.anchor.clone(),
        why,
    };
    let mut lines = text.lines();
    let found = lines
        .by_ref()
        .any(|line| line.starts_with('#') && anchor_of(line.trim_start_matches('#')) == d.anchor);
    if !found {
        return Err(entry("no such heading"));
    }
    if lines
        .take_while(|line| !line.starts_with('#'))
        .any(|line| line.starts_with("Status: DEVIATION"))
    {
        Ok(())
    } else {
        Err(entry("the entry has no `Status: DEVIATION` line"))
    }
}

fn read(path: &Path) -> Result<String, DeviationError> {
    std::fs::read_to_string(path).map_err(|e| DeviationError::Unreadable {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

/// The suite's parsed `deviations` file, or none when its manifest names no file.
///
/// # Errors
/// When the file is unreadable or malformed.
pub fn load(suite: &Suite) -> Result<Vec<Deviation>, DeviationError> {
    match &suite.manifest.deviations {
        Some(file) => parse(&read(&suite.dir.join(file))?),
        None => Ok(Vec::new()),
    }
}

/// Loads the suite's `deviations` file and checks every line against the repository: the case
/// file exists and the cited semantics entry is an accepted DEVIATION (decision 0036). Whether
/// the cell is scored is checked when the case runs.
///
/// # Errors
/// The first unusable line.
pub fn load_checked(suite: &Suite, repo_root: &Path) -> Result<Vec<Deviation>, DeviationError> {
    let listed = load(suite)?;
    let root = suite.root();
    for d in &listed {
        if !root.join(&d.case).is_file() {
            return Err(DeviationError::MissingCase(d.case.clone()));
        }
        check_entry(d, repo_root)?;
    }
    Ok(listed)
}

/// Scores a listed cell whose by-design value is `pinned` (decision 0036).
#[must_use]
pub fn classify(pinned: &Value, expected: &Value, actual: &Value) -> Outcome {
    if compare(expected, actual).is_pass() {
        Outcome::StalePass
    } else if compare(pinned, actual).is_pass() {
        Outcome::Deviation
    } else {
        Outcome::Failure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = "cases/lifted-range-rows.case.json\tSheet1!C1\tdecision 0014\t\
        docs/excel-semantics/arrays.md#array-results-above-1048576-cells-are-error\t#ERROR!";

    #[test]
    fn parses_lines_and_skips_comments() {
        let parsed = parse(&format!("# why\n\n{LINE}\n")).expect("parse");
        assert_eq!(parsed.len(), 1);
        let d = &parsed[0];
        assert_eq!(d.case, "cases/lifted-range-rows.case.json");
        let Pin::Cell { sheet, addr, value } = &d.pin else {
            panic!("a cell line: {d:?}");
        };
        assert_eq!(
            (sheet.as_str(), addr.to_string()),
            ("Sheet1", "C1".to_string())
        );
        assert_eq!(d.decision, "0014");
        assert_eq!(d.semantics_file, "docs/excel-semantics/arrays.md");
        assert_eq!(*value, Value::Error("#ERROR!".into()));
    }

    #[test]
    fn parses_a_whole_case_refusal() {
        let line = "a.case.json\t*\tdecision 0014\tdocs/excel-semantics/syntax.md#x\t\
                    refused \"Formula is nested too deeply\"";
        let parsed = parse(line).expect("parse");
        assert_eq!(
            parsed[0].pin,
            Pin::Refusal("Formula is nested too deeply".into())
        );
    }

    #[test]
    fn parses_every_kind_of_pinned_value() {
        for (text, value) in [
            ("#N/A", Value::Error("#N/A".into())),
            ("TRUE", Value::Bool(true)),
            ("\"a b\"", Value::Text("a b".into())),
            ("-1.5", Value::Number(-1.5)),
        ] {
            assert_eq!(parse_value(text), Some(value), "{text}");
        }
        for text in ["", "#", "true", "abc", "inf", "NaN"] {
            assert_eq!(parse_value(text), None, "{text}");
        }
    }

    #[test]
    fn rejects_malformed_lines() {
        let sem = "docs/excel-semantics/arrays.md#x";
        for bad in [
            format!("a.json\tSheet1!C1\tdecision 0014\t{sem}"),
            format!("a.json\tC1\tdecision 0014\t{sem}\t#ERROR!"),
            format!("../a.json\tS!C1\tdecision 0014\t{sem}\t#ERROR!"),
            format!("a.json\tS!C1\t0014\t{sem}\t#ERROR!"),
            format!("a.json\tS!C1\tdecision 14\t{sem}\t#ERROR!"),
            "a.json\tS!C1\tdecision 0014\tdocs/other/arrays.md#x\t#ERROR!".to_string(),
            "a.json\tS!C1\tdecision 0014\tdocs/excel-semantics/arrays.md\t#ERROR!".to_string(),
            "a.json\tS!C1\tdecision 0014\tdocs/excel-semantics/../x.md#y\t#ERROR!".to_string(),
            format!("a.json\tS!C1\tdecision 0014\t{sem}\tnot-a-value"),
            format!("a.json\t*\tdecision 0014\t{sem}\t#ERROR!"),
            format!("a.json\t*\tdecision 0014\t{sem}\trefused \"\""),
            format!("a.json\t*\tdecision 0014\t{sem}\trefused deep"),
            format!("a.json\tS!C1\tdecision 0014\t{sem}\trefused \"deep\""),
        ] {
            assert!(parse(&bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_cited_entry_must_be_a_deviation() {
        let root = std::env::temp_dir().join(format!("deviations-{}", std::process::id()));
        let dir = root.join("docs/excel-semantics");
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(
            dir.join("arrays.md"),
            "# Arrays\n## Big arrays are #ERROR! (1,048,576)\nRule: x\nStatus: DEVIATION (decision 0014)\n\
             ## Plain rule\nRule: y\nStatus: VERIFIED\n## Next\nStatus: DEVIATION (decision 0014)\n",
        )
        .expect("write");
        let at = |anchor: &str| {
            parse(&format!(
                "a.json\tS!A1\tdecision 0014\tdocs/excel-semantics/arrays.md#{anchor}\t#ERROR!"
            ))
            .expect("parse")
            .remove(0)
        };
        assert_eq!(
            check_entry(&at("big-arrays-are-error-1048576"), &root),
            Ok(())
        );
        // a VERIFIED entry, even when the next section is a DEVIATION
        assert!(check_entry(&at("plain-rule"), &root).is_err());
        assert!(check_entry(&at("no-such-heading"), &root).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn load_checked_needs_the_case_file() {
        let root = std::env::temp_dir().join(format!("deviations-case-{}", std::process::id()));
        std::fs::create_dir_all(root.join("docs/excel-semantics")).expect("dir");
        std::fs::create_dir_all(root.join("s/cases")).expect("dir");
        std::fs::write(
            root.join("docs/excel-semantics/arrays.md"),
            "## X\nStatus: DEVIATION (decision 0014)\n",
        )
        .expect("write");
        std::fs::write(
            root.join("s/deviations"),
            "a.case.json\tS!A1\tdecision 0014\tdocs/excel-semantics/arrays.md#x\t#ERROR!\n",
        )
        .expect("write");
        let suite = Suite {
            manifest: toml::from_str(
                "name = \"s\"\nkind = \"spec\"\nlicense = \"MIT\"\nroot = \"cases\"\n\
                 deviations = \"deviations\"\n",
            )
            .expect("manifest"),
            dir: root.join("s"),
            cache_dir: std::path::PathBuf::new(),
        };
        let missing = load_checked(&suite, &root);
        std::fs::write(root.join("s/cases/a.case.json"), "{}").expect("write");
        let present = load_checked(&suite, &root);
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            missing,
            Err(DeviationError::MissingCase("a.case.json".into()))
        );
        assert_eq!(present.map(|d| d.len()), Ok(1));
    }

    #[test]
    fn scores_the_three_outcomes() {
        let Pin::Cell { value: pinned, .. } = parse(LINE).expect("parse").remove(0).pin else {
            panic!("a cell line");
        };
        let excel = Value::Number(1000.0);
        let actual = Value::Error("#ERROR!".into());
        assert_eq!(classify(&pinned, &excel, &actual), Outcome::Deviation);
        assert_eq!(classify(&pinned, &excel, &excel), Outcome::StalePass);
        assert_eq!(
            classify(&pinned, &excel, &Value::Number(999.0)),
            Outcome::Failure
        );
    }
}
