//! Displayed-text expectations (T-1610, decision 0068): a spec case lists cells in `text`, its
//! oracle sidecar holds Excel's displayed text for them (`Range.Text`, read by the backend after
//! the recalculation), and the eval compares that with the engine's formatted text. Text is scored
//! as a verdict of its own beside the value's, so value scoring, baselines and the gate do not
//! change; the report shows a text pass rate per suite.

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
}
