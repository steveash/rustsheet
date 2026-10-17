//! Displayed-text expectations (T-1610, decision 0068): a spec case lists cells in `text`, its
//! oracle sidecar holds Excel's displayed text for them (`Range.Text`, read by the backend after
//! the recalculation), and the eval compares that with the engine's formatted text. Text is scored
//! as a verdict of its own beside the value's, so value scoring does not change; the report shows
//! a text pass rate per suite, and the baseline gates text cells on `TC`/`TF` lines of their own
//! (decision 0070).

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::case::{load_oracle, split_qualified, CaseSpec};
use crate::cellref::Addr;
use crate::engine::Evaluated;
use crate::report::SuiteSummary;
use crate::runner::CaseResult;
use crate::suite::CaseFile;

/// One cell whose displayed text is scored.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TextExpectation {
    sheet: String,
    addr: Addr,
    expected: String,
}

/// The displayed-text verdict of one cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextResult {
    pub sheet: String,
    pub cell: String,
    /// Excel's displayed text.
    pub expected: String,
    /// The engine's formatted text, `None` when it could not format the cell.
    pub actual: Option<String>,
    pub pass: bool,
}

/// The text expectations of a spec case: each cell listed in `text` that the oracle sidecar
/// holds a text for. A listed cell with no text yet is pending, so it is not scored.
///
/// # Errors
/// When a listed cell is not a cell reference or the sidecar cannot be read.
pub(crate) fn expectations(
    spec: &CaseSpec,
    case: &CaseFile,
) -> Result<Vec<TextExpectation>, String> {
    if spec.text.is_empty() {
        return Ok(Vec::new());
    }
    let oracle = load_oracle(case)?;
    let mut out = Vec::with_capacity(spec.text.len());
    for key in &spec.text {
        let bad = || format!("bad text cell key {key}");
        let (sheet, a1) = split_qualified(key).ok_or_else(bad)?;
        let addr = Addr::parse(&a1).ok_or_else(bad)?;
        let expected = oracle.as_ref().and_then(|o| {
            let cells = o.text.get(&sheet)?;
            // The backend writes the A1 key as the case lists it, `$` and case aside.
            cells
                .iter()
                .find(|(k, _)| Addr::parse(k) == Some(addr))
                .map(|(_, text)| text.clone())
        });
        if let Some(expected) = expected {
            out.push(TextExpectation {
                sheet,
                addr,
                expected,
            });
        }
    }
    Ok(out)
}

/// Scores each expectation against the engine's formatted text: a pass is the exact string.
pub(crate) fn score(
    evaluated: &dyn Evaluated,
    expectations: &[TextExpectation],
) -> Vec<TextResult> {
    expectations
        .iter()
        .map(|e| {
            let actual = evaluated.text(&e.sheet, e.addr);
            TextResult {
                sheet: e.sheet.clone(),
                cell: e.addr.to_string(),
                expected: e.expected.clone(),
                pass: actual.as_deref() == Some(e.expected.as_str()),
                actual,
            }
        })
        .collect()
}

/// The report's "Displayed text" section: each suite's text pass rate, then every failing text
/// cell (`cases` keeps only failing ones). Nothing when no suite scored a text.
pub(crate) fn write_markdown(suites: &[SuiteSummary], cases: &[CaseResult], md: &mut String) {
    if suites.iter().all(|s| s.text_cells == 0) {
        return;
    }
    let _ = writeln!(md, "\n## Displayed text\n");
    let _ = writeln!(md, "| suite | text cells | pass | text pass % |");
    let _ = writeln!(md, "|---|---|---|---|");
    for s in suites.iter().filter(|s| s.text_cells > 0) {
        #[allow(clippy::cast_precision_loss)] // counts far below 2^52
        let rate = 100.0 * s.text_passed as f64 / s.text_cells as f64;
        let _ = writeln!(
            md,
            "| {} | {} | {} | {rate:.2} |",
            s.name, s.text_cells, s.text_passed
        );
    }
    let failing: Vec<(&str, &TextResult)> = cases
        .iter()
        .flat_map(|c| {
            c.text
                .iter()
                .filter(|t| !t.pass)
                .map(|t| (c.id.as_str(), t))
        })
        .collect();
    if !failing.is_empty() {
        let _ = writeln!(md, "\n| case | cell | Excel | ours |\n|---|---|---|---|");
        for (id, t) in failing {
            let ours = t.actual.as_deref().unwrap_or("(none)");
            let _ = writeln!(
                md,
                "| {id} | {}!{} | `{}` | `{ours}` |",
                t.sheet, t.cell, t.expected
            );
        }
    }
}

/// The `show` command's displayed-text lines for one case: the pass count, then each failing text
/// cell (every cell with `all`). Nothing when the case scored no text.
pub fn write_case(text: &[TextResult], all: bool, out: &mut String) {
    if text.is_empty() {
        return;
    }
    let passed = text.iter().filter(|t| t.pass).count();
    let _ = writeln!(out, "text: {passed}/{} passed", text.len());
    for t in text.iter().filter(|t| all || !t.pass) {
        let ours = t.actual.as_deref().unwrap_or("(none)");
        let _ = writeln!(
            out,
            "{}!{:<14} {}\n    Excel text: {}\n    our text  : {ours}",
            t.sheet,
            t.cell,
            if t.pass { "Pass" } else { "Fail" },
            t.expected
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    struct Fake;

    impl Evaluated for Fake {
        fn value(&self, _sheet: &str, _addr: Addr) -> Option<Value> {
            Some(Value::Empty)
        }
        fn stored_cells(&self, _sheet: &str) -> Vec<Addr> {
            Vec::new()
        }
        fn text(&self, sheet: &str, addr: Addr) -> Option<String> {
            (sheet == "S").then(|| format!("t{addr}"))
        }
    }

    fn expectation(sheet: &str, a1: &str, expected: &str) -> TextExpectation {
        TextExpectation {
            sheet: sheet.into(),
            addr: Addr::parse(a1).unwrap_or_else(|| panic!("{a1}")),
            expected: expected.into(),
        }
    }

    #[test]
    fn scores_the_exact_displayed_text() {
        let results = score(
            &Fake,
            &[
                expectation("S", "A1", "tA1"),
                expectation("S", "B2", "x"),
                expectation("Other", "A1", "tA1"),
            ],
        );
        let verdicts: Vec<_> = results.iter().map(|r| (r.cell.as_str(), r.pass)).collect();
        assert_eq!(verdicts, [("A1", true), ("B2", false), ("A1", false)]);
        assert_eq!(results[2].actual, None);
    }

    #[test]
    fn show_lists_the_failing_text_cells_and_the_pass_count() {
        let results = score(
            &Fake,
            &[expectation("S", "A1", "tA1"), expectation("S", "B2", "x")],
        );
        let mut out = String::new();
        write_case(&results, false, &mut out);
        assert!(out.starts_with("text: 1/2 passed\n"), "{out}");
        assert!(
            out.contains("S!B2") && out.contains("Excel text: x"),
            "{out}"
        );
        assert!(out.contains("our text  : tB2"), "{out}");
        assert!(!out.contains("S!A1"), "{out}");
        let mut all = String::new();
        write_case(&results, true, &mut all);
        assert!(all.contains("S!A1"), "{all}");
        let mut none = String::new();
        write_case(&[], true, &mut none);
        assert!(none.is_empty());
    }

    #[test]
    fn listed_cells_without_oracle_text_are_pending() {
        let dir = std::env::temp_dir().join(format!("text-expect-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
        let case = CaseFile {
            id: "t.case.json".into(),
            path: dir.join("t.case.json"),
            sidecar: dir.join("t.oracle.json"),
        };
        let spec: CaseSpec = serde_json::from_str(
            r#"{"format":"rustsheet-case/v1","sheets":[],"text":["S!A1","'S'!$B$2"]}"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(expectations(&spec, &case), Ok(Vec::new()));
        std::fs::write(
            &case.sidecar,
            r#"{"format":"rustsheet-oracle/v1","source":{"kind":"excel-graph"},"sheets":{},
                "text":{"S":{"B2":"1.50"}}}"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            expectations(&spec, &case),
            Ok(vec![expectation("S", "B2", "1.50")])
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn report_lists_the_text_pass_rate_and_failing_cells() {
        let mut md = String::new();
        write_markdown(&[SuiteSummary::default()], &[], &mut md);
        assert_eq!(md, "");
        let suite = SuiteSummary {
            name: "probes".into(),
            text_cells: 4,
            text_passed: 3,
            ..SuiteSummary::default()
        };
        let case: CaseResult = serde_json::from_value(serde_json::json!({
            "id": "probes/t.case.json", "suite": "probes", "status": "fail",
            "expected_source": "excel-graph", "checked": 0, "passed": 0,
            "skipped_volatile": 0, "millis": 0, "cells": [],
            "text": [{"sheet": "S", "cell": "A2", "expected": "12 1/2", "actual": " 1/2",
                      "pass": false}]
        }))
        .unwrap_or_else(|e| panic!("{e}"));
        write_markdown(&[suite], &[case], &mut md);
        assert!(md.contains("| probes | 4 | 3 | 75.00 |"), "{md}");
        assert!(
            md.contains("| probes/t.case.json | S!A2 | `12 1/2` | ` 1/2` |"),
            "{md}"
        );
    }

    #[test]
    fn a_malformed_text_cell_is_an_error() {
        let case = CaseFile {
            id: "t.case.json".into(),
            path: "t.case.json".into(),
            sidecar: "missing.oracle.json".into(),
        };
        let spec: CaseSpec =
            serde_json::from_str(r#"{"format":"rustsheet-case/v1","sheets":[],"text":["A1"]}"#)
                .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            expectations(&spec, &case),
            Err("bad text cell key A1".to_string())
        );
    }
}
