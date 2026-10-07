//! On-disk formats: spec cases (`*.case.json`) and oracle sidecars (`*.oracle.json`).
//!
//! See `docs/architecture/eval-harness.md` for the normative description.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::cellref::Addr;
use crate::value::Value;

pub const CASE_FORMAT: &str = "rustsheet-case/v1";
pub const ORACLE_FORMAT: &str = "rustsheet-oracle/v1";

/// A cell input in a spec case.
///
/// JSON numbers are numbers, JSON booleans are booleans, strings starting with
/// `=` are formulas, other strings are literal text. Use the tagged form
/// (`{"t":"s","v":"=not a formula"}`, `{"t":"e","v":"#N/A"}`) for anything else. An error may
/// carry `"stored":true`, which only changes how spec2xlsx writes it (a stored constant, T-0999);
/// the engine adapter ignores it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellInput {
    Number(f64),
    Bool(bool),
    /// Formula (leading `=`) or literal text.
    Str(String),
    Typed(Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSpec {
    pub name: String,
    /// A1 address → input.
    pub cells: BTreeMap<String, CellInput>,
    /// Formula cells (A1 addresses in `cells`) that spec2xlsx writes as legacy, non-array formulas
    /// (no dynamic-array metadata; T-1077). The engine adapter ignores them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legacy_formulas: Vec<String>,
}

/// A self-contained workbook test case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseSpec {
    pub format: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub sheets: Vec<SheetSpec>,
    /// Workbook-scoped defined names: name → formula (e.g. `"=Sheet1!$A$1:$B$2"`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub names: BTreeMap<String, String>,
    /// Cells to score: `Sheet!A1` → expected value. Missing/empty means "pending oracle"
    /// (unless a sidecar oracle file supplies them).
    #[serde(default)]
    pub expected: BTreeMap<String, Value>,
    /// Where `expected` came from (`excel-graph`, `excel-desktop`, `hand`, ...).
    #[serde(default)]
    pub expected_source: Option<String>,
    /// If set, only these cells are scored (otherwise every key in `expected`).
    #[serde(default)]
    pub check: Vec<String>,
}

/// Provenance of oracle values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OracleSource {
    /// `excel-graph` | `excel-desktop` | `excel-cached` | `libreoffice` | `hand`.
    pub kind: String,
    #[serde(default)]
    pub application: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    /// RFC 3339 timestamp.
    #[serde(default)]
    pub produced_at: Option<String>,
    /// sha256 of the workbook bytes the oracle evaluated.
    #[serde(default)]
    pub input_sha256: Option<String>,
}

/// Whether an oracle `source.kind` string (see [`OracleSource::kind`]) is Excel evidence rather
/// than hand-written or engine-derived values.
///
/// Only call this on oracle `source.kind` values: case-level labels such as
/// `excel-cached (<app>)` also start with "excel" even when `<app>` is not Excel.
pub(crate) fn is_excel_evidence(expected_source: &str) -> bool {
    expected_source.starts_with("excel")
}

/// Oracle results for one workbook: sheet → A1 → value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OracleFile {
    pub format: String,
    pub source: OracleSource,
    pub sheets: BTreeMap<String, BTreeMap<String, Value>>,
}

/// Splits `Sheet!A1` / `'My Sheet'!A1` into (sheet, cell).
#[must_use]
pub fn split_qualified(s: &str) -> Option<(String, String)> {
    let (sheet, cell) = s.rsplit_once('!')?;
    let sheet = sheet
        .strip_prefix('\'')
        .and_then(|x| x.strip_suffix('\''))
        .map_or_else(|| sheet.to_string(), |x| x.replace("''", "'"));
    Some((sheet, cell.to_string()))
}

/// The canonical expectation key for a cell: `'Sheet'!A1` (sheet always quoted, `'` doubled).
#[must_use]
pub fn cell_key(sheet: &str, addr: Addr) -> String {
    format!("'{}'!{addr}", sheet.replace('\'', "''"))
}

/// Canonicalises a `Sheet!A1` / `'Sheet'!$A$1` key; `None` if it is not a cell reference.
#[must_use]
pub fn canonical_key(key: &str) -> Option<String> {
    let (sheet, a1) = split_qualified(key)?;
    Some(cell_key(&sheet, Addr::parse(&a1)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_case_inputs() {
        let json = r##"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "A1":1,"A2":"x","A3":"=A1+1","A4":true,"A5":{"t":"e","v":"#N/A"}}}],
            "expected":{"S!A3":{"t":"n","v":2}}}"##;
        let spec: Result<CaseSpec, _> = serde_json::from_str(json);
        let spec = spec.unwrap_or_else(|e| panic!("{e}"));
        let cells = &spec.sheets[0].cells;
        assert_eq!(cells["A1"], CellInput::Number(1.0));
        assert_eq!(cells["A3"], CellInput::Str("=A1+1".into()));
        assert_eq!(cells["A4"], CellInput::Bool(true));
        assert_eq!(cells["A5"], CellInput::Typed(Value::Error("#N/A".into())));
    }

    #[test]
    fn splits_qualified_refs() {
        assert_eq!(split_qualified("S!A1"), Some(("S".into(), "A1".into())));
        assert_eq!(
            split_qualified("'It''s'!B2"),
            Some(("It's".into(), "B2".into()))
        );
    }
}
