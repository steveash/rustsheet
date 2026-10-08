//! Per-cell replacements for Excel-cached xlsx values that are not en-US Excel evidence
//! (decision 0032): a suite's `cached_overrides` file names a scored cell and the committed
//! oracle value that replaces its cached expectation, so a known non-en-US cached value is scored
//! against en-US Excel instead of becoming a baseline "regression" or a dropped cell.
//!
//! File format, one override per line, tab-separated; `#` starts a comment line:
//!
//! ```text
//! <case path relative to the suite root>  <Sheet!A1>  <oracle sidecar, relative to eval/suites>  <Sheet!A1>
//! ```
//!
//! The replacement must come from a golden oracle sidecar (`excel-graph` or `excel-desktop`):
//! an override never carries a hand-written value. The same file tells `rustsheet_xlsx`'s generated tests which
//! cached values not to compare (its `build.rs`).

use std::path::{Component, Path};

use crate::case::{split_qualified, OracleFile};
use crate::cellref::Addr;
use crate::value::Value;

/// Oracle sources that may replace a cached value: a live Excel, never cached values or an
/// approximate backend.
const GOLDEN_KINDS: [&str; 2] = [crate::case::GRAPH_ORACLE, "excel-desktop"];

/// One `cached_overrides` line: a cached cell and the golden cell that replaces it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedOverride {
    /// Case path relative to the suite root (`DATE_AND_TIME/DATEVALUE.xlsx`).
    pub case: String,
    pub sheet: String,
    pub addr: Addr,
    /// Oracle sidecar relative to the suites directory (`probes/cases/x.oracle.json`).
    pub evidence: String,
    pub evidence_sheet: String,
    pub evidence_addr: Addr,
}

/// A qualified `Sheet!A1` cell of a per-cell suite file.
pub(crate) fn parse_cell(text: &str) -> Option<(String, Addr)> {
    let (sheet, a1) = split_qualified(text)?;
    Some((sheet, Addr::parse(&a1)?))
}

/// Whether `path` is relative and stays inside its base (no root, `..` or `.`).
pub(crate) fn is_inner_relative(path: &str) -> bool {
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

/// Parses a `cached_overrides` file.
///
/// # Errors
/// Names the first malformed line.
pub fn parse(text: &str) -> Result<Vec<CachedOverride>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let bad = |why: &str| format!("cached_overrides line {}: {why}", n + 1);
        let fields: Vec<&str> = line.split('\t').collect();
        let [case, cell, evidence, evidence_cell] = fields.as_slice() else {
            return Err(bad("expected 4 tab-separated fields"));
        };
        let (sheet, addr) = parse_cell(cell).ok_or_else(|| bad("bad cell"))?;
        let (evidence_sheet, evidence_addr) =
            parse_cell(evidence_cell).ok_or_else(|| bad("bad evidence cell"))?;
        if !is_inner_relative(case) || !is_inner_relative(evidence) {
            return Err(bad("paths must be relative, without `..`"));
        }
        if !evidence.ends_with(".oracle.json") {
            return Err(bad("evidence must be an oracle sidecar (*.oracle.json)"));
        }
        out.push(CachedOverride {
            case: (*case).to_string(),
            sheet,
            addr,
            evidence: (*evidence).to_string(),
            evidence_sheet,
            evidence_addr,
        });
    }
    Ok(out)
}

/// The golden value an override points at.
///
/// # Errors
/// When the sidecar is unreadable, not from an Excel oracle, or lacks the cell.
pub fn evidence_value(o: &CachedOverride, suites_dir: &Path) -> Result<Value, String> {
    let path = suites_dir.join(&o.evidence);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let oracle: OracleFile =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if !GOLDEN_KINDS.contains(&oracle.source.kind.as_str()) {
        return Err(format!(
            "{}: override evidence must come from an Excel oracle, not {:?}",
            o.evidence, oracle.source.kind
        ));
    }
    oracle
        .sheets
        .get(&o.evidence_sheet)
        .and_then(|cells| cells.get(&o.evidence_addr.to_string()))
        .cloned()
        .ok_or_else(|| {
            format!(
                "{} has no value for {}!{}",
                o.evidence, o.evidence_sheet, o.evidence_addr
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str =
        "DATE/DATEVALUE.xlsx\tDATEVALUE!B48\tprobes/cases/date-text-locale.oracle.json\tSheet1!B1";

    #[test]
    fn parses_lines_and_skips_comments() {
        let parsed = parse(&format!("# why\n\n{LINE}\n")).expect("parse");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].case, "DATE/DATEVALUE.xlsx");
        assert_eq!(parsed[0].sheet, "DATEVALUE");
        assert_eq!(parsed[0].addr.to_string(), "B48");
        assert_eq!(parsed[0].evidence_addr.to_string(), "B1");
    }

    #[test]
    fn rejects_malformed_lines() {
        for bad in [
            "a.xlsx\tSheet1!B1\tp.oracle.json",
            "a.xlsx\tB1\tp.oracle.json\tSheet1!B1",
            "../a.xlsx\tS!B1\tp.oracle.json\tS!B1",
            "a.xlsx\tS!B1\t/abs/p.oracle.json\tS!B1",
            "a.xlsx\tS!B1\tprobes/cases/p.case.json\tS!B1",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn evidence_must_be_an_excel_oracle() {
        let dir = std::env::temp_dir().join(format!("overrides-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let sidecar = |kind: &str| {
            format!(
                r#"{{"format":"rustsheet-oracle/v1","source":{{"kind":"{kind}"}},
                "sheets":{{"Sheet1":{{"B1":{{"t":"n","v":30.0}}}}}}}}"#
            )
        };
        let o = &parse("a.xlsx\tS!A1\tx.oracle.json\tSheet1!B1").expect("parse")[0];
        std::fs::write(dir.join("x.oracle.json"), sidecar("excel-graph")).expect("write");
        assert_eq!(evidence_value(o, &dir), Ok(Value::Number(30.0)));
        for kind in ["libreoffice", "excel-cached", "hand"] {
            std::fs::write(dir.join("x.oracle.json"), sidecar(kind)).expect("write");
            assert!(evidence_value(o, &dir).is_err(), "{kind}");
        }
        let missing = &parse("a.xlsx\tS!A1\tx.oracle.json\tSheet1!C9").expect("parse")[0];
        assert!(evidence_value(missing, &dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
