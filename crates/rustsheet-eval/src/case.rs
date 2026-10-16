//! On-disk formats: spec cases (`*.case.json`) and oracle sidecars (`*.oracle.json`).
//!
//! See `docs/architecture/eval-harness.md` for the normative description.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::cellref::Addr;
use crate::suite::CaseFile;
use crate::value::Value;

pub const CASE_FORMAT: &str = "rustsheet-case/v1";
pub const ORACLE_FORMAT: &str = "rustsheet-oracle/v1";

/// A cell input in a spec case.
///
/// JSON numbers are numbers, JSON booleans are booleans, strings starting with
/// `=` are formulas, other strings are literal text. Use the tagged form
/// (`{"t":"s","v":"=not a formula"}`, `{"t":"e","v":"#N/A"}`) for anything else. An error may
/// carry `"stored":true`: spec2xlsx writes it as a stored `t="e"` constant (T-0999) and the engine
/// adapter enters it as one, as xlsx import does (T-1221), not as typed input. `{"entered":"(1)"}`
/// is text entered as a user types it, so Excel picks the value and number format (T-0280).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellInput {
    Number(f64),
    Bool(bool),
    /// Formula (leading `=`) or literal text.
    Str(String),
    /// `{"t":"e","v":"#X","stored":true}`: an error constant. Must precede `Typed`, which would
    /// otherwise match it and drop `stored`.
    StoredError(StoredError),
    Typed(Value),
    /// Typed entry: spec2xlsx leaves the cell blank and records it for the oracle backend, which
    /// writes it into the cell through Excel's own input parser.
    Entered {
        entered: String,
    },
}

/// A stored error constant (`{"t":"e","v":"#N/A","stored":true}`); only that exact shape matches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredError {
    t: ErrorTag,
    /// The error name as written, e.g. `#N/A`.
    pub v: String,
    stored: StoredTrue,
}

impl StoredError {
    /// A stored error constant named `name`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            t: ErrorTag::E,
            v: name.into(),
            stored: StoredTrue,
        }
    }
}

/// The constant `"e"` of an error cell's `t` key.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum ErrorTag {
    #[serde(rename = "e")]
    E,
}

/// The constant `true`: `stored: false` is not a stored error.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "bool", into = "bool")]
struct StoredTrue;

impl TryFrom<bool> for StoredTrue {
    type Error = &'static str;

    fn try_from(stored: bool) -> Result<Self, Self::Error> {
        stored.then_some(Self).ok_or("expected stored = true")
    }
}

impl From<StoredTrue> for bool {
    fn from(_: StoredTrue) -> Self {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSpec {
    pub name: String,
    /// A1 address → input.
    pub cells: BTreeMap<String, CellInput>,
    /// Formula cells (A1 addresses in `cells`) that spec2xlsx writes as legacy, non-array formulas
    /// (no dynamic-array metadata; T-1077). The engine adapter enters them as
    /// `Input::LegacyFormula`, with the implicit intersections xlsx import adds (T-1096).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legacy_formulas: Vec<String>,
    /// Formula cells that spec2xlsx writes without `_xlfn.` prefixes, as typed: a probe of how a
    /// bare newer function name loads from a file (T-0416). The engine adapter ignores the list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bare_function_names: Vec<String>,
    /// 1-based rows written hidden (T-1036). On a sheet with a filtered table every hidden row
    /// counts as filtered, as Excel reads such a file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden_rows: Vec<u32>,
    /// Tables (`ListObject`s) on the sheet, header row first (T-1036).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tables: Vec<TableSpec>,
    /// What-if data tables (`TABLE()`), written by spec2xlsx as `<f t="dataTable">` (T-0428). The
    /// engine adapter ignores them until Phase 2 implements data tables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_tables: Vec<DataTableSpec>,
    /// Pivot tables, written by spec2xlsx with their rendered cells, cache and definition
    /// (`tools/src/rustsheet_tools/pivot.py`, T-1705). The engine has no pivot tables, so the
    /// adapter ignores them: a case scores only the GETPIVOTDATA formulas that read them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pivot_tables: Vec<PivotTableSpec>,
    /// A1 address → number format code (`"[h]:mm"`), applied to that cell whatever it holds
    /// (T-1268). spec2xlsx writes it as the cell's `numFmt`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub number_formats: BTreeMap<String, String>,
}

/// A table in a spec sheet. spec2xlsx gives a filtered table a values filter that keeps exactly
/// its visible data rows, so the sheet's `hidden_rows` inside it are the rows the filter hides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableSpec {
    pub name: String,
    /// A1 range, header row included (`A1:B9`).
    #[serde(rename = "ref")]
    pub range: String,
    #[serde(default)]
    pub filter: bool,
    /// The last row of `range` is the table's totals row (T-1607). The facade cannot add one yet
    /// (TODO(T-1671)), so the engine adapter registers the table without it: data references read
    /// right, `#Totals` and `#All` do not.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub totals_row: bool,
}

/// A what-if data table in a spec sheet: `ref` holds its result cells; `row_input` alone is a
/// one-variable row table, `column_input` alone a column table, both a two-variable table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTableSpec {
    /// A1 range of the result cells (`B4:B6`).
    #[serde(rename = "ref")]
    pub range: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_input: Option<String>,
}

/// A pivot table in a spec sheet (T-1705): `row` (and optionally `column`) are text fields of the
/// `source` range's header row, `value` a number field aggregated by `function` (`sum` when absent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PivotTableSpec {
    pub name: String,
    /// A1 range with the field names in its first row, optionally sheet-qualified (`Data!A1:C7`).
    pub source: String,
    /// Top-left cell of the rendered pivot.
    pub location: String,
    pub row: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
}

/// A workbook's calculation settings (`<calcPr>`), written by spec2xlsx (T-0428). The engine
/// adapter runs an iterative case's three oracle calculations (T-0429) but does not pass the
/// settings on until Phase 2 implements iterative calculation (T-0401).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalculationSpec {
    /// Iterative calculation on (`iterate="1"`).
    #[serde(default)]
    pub iterate: bool,
    /// Maximum iterations per calculation (`iterateCount`; Excel's default is 100).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterate_count: Option<u32>,
    /// Maximum change between iterations (`iterateDelta`; Excel's default is 0.001).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterate_delta: Option<f64>,
}

/// A sheet edit made after a case loads and before it is recalculated (T-1608): spec2xlsx records
/// it for the oracle backend, which replays it in Excel, and the engine adapter applies it through
/// the edit API. `{"op":"add","sheet":"New"}` appends a sheet, `{"op":"rename","sheet":"Data",
/// "to":"Inputs"}` renames one, `{"op":"move","sheet":"Data","to":0}` moves one to a 0-based
/// position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
pub enum SheetOp {
    Add { sheet: String },
    Rename { sheet: String, to: String },
    Move { sheet: String, to: u32 },
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
    /// The workbook's date system, `1900` (the default) or `1904` (`<workbookPr date1904="1"/>`;
    /// SPEC-0005, T-1268). Any other value is refused when the case is built.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_system: Option<u16>,
    /// Calculation settings such as iterative calculation; Excel's defaults when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calculation: Option<CalculationSpec>,
    /// Sheet edits applied in order after the workbook loads, before recalculation (T-1608).
    /// Cell keys in `expected` and `check` use the sheet names after the edits.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sheet_ops: Vec<SheetOp>,
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
    /// Cells (`Sheet!A1`) that call a volatile function, or read a volatile cell, yet cannot
    /// change value between runs (`RANDBETWEEN(5,5)`, `ISNUMBER(NOW())`). They are scored instead
    /// of skipped as volatile (T-1283); every listed cell must be a volatile formula cell.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariant: Vec<String>,
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

/// The oracle kind of the Microsoft Graph backend.
pub(crate) const GRAPH_ORACLE: &str = "excel-graph";

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

/// The cells a spec case holds an expectation for, as `Sheet!A1` (the form of
/// `runner::CellResult::key`): the keys of its `expected` and of its oracle sidecar.
///
/// # Errors
/// When the case or its sidecar cannot be read, or names a malformed cell.
pub fn expectation_cells(case: &CaseFile) -> Result<BTreeSet<String>, String> {
    let text = std::fs::read_to_string(&case.path).map_err(|e| e.to_string())?;
    let spec: CaseSpec = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut keys = Vec::new();
    for key in spec.expected.keys() {
        keys.push(split_qualified(key).ok_or_else(|| format!("bad cell key {key}"))?);
    }
    if let Some(o) = load_oracle(case)? {
        for (sheet, cells) in o.sheets {
            keys.extend(cells.into_keys().map(|a1| (sheet.clone(), a1)));
        }
    }
    keys.into_iter()
        .map(|(sheet, a1)| {
            let addr = Addr::parse(&a1).ok_or_else(|| format!("bad cell key {sheet}!{a1}"))?;
            Ok(format!("{sheet}!{addr}"))
        })
        .collect()
}

/// The case's oracle sidecar, or `None` when it has none.
///
/// # Errors
/// When the sidecar cannot be read or parsed.
pub fn load_oracle(case: &CaseFile) -> Result<Option<OracleFile>, String> {
    let path = &case.sidecar;
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_case_inputs() {
        let json = r##"{"format":"rustsheet-case/v1","sheets":[{"name":"S","cells":{
            "A1":1,"A2":"x","A3":"=A1+1","A4":true,"A5":{"t":"e","v":"#N/A"},"A6":{"entered":"(1)"}}}],
            "expected":{"S!A3":{"t":"n","v":2}}}"##;
        let spec: Result<CaseSpec, _> = serde_json::from_str(json);
        let spec = spec.unwrap_or_else(|e| panic!("{e}"));
        let cells = &spec.sheets[0].cells;
        assert_eq!(cells["A1"], CellInput::Number(1.0));
        assert_eq!(cells["A3"], CellInput::Str("=A1+1".into()));
        assert_eq!(cells["A4"], CellInput::Bool(true));
        assert_eq!(cells["A5"], CellInput::Typed(Value::Error("#N/A".into())));
        assert_eq!(
            cells["A6"],
            CellInput::Entered {
                entered: "(1)".into()
            }
        );
    }

    #[test]
    fn stored_errors_are_told_apart_from_typed_ones() {
        let parse = |json: &str| -> CellInput {
            serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}"))
        };
        assert_eq!(
            parse(r##"{"t":"e","v":"#N/A","stored":true}"##),
            CellInput::StoredError(StoredError::new("#N/A"))
        );
        for json in [
            r##"{"t":"e","v":"#N/A"}"##,
            r##"{"t":"e","v":"#N/A","stored":false}"##,
        ] {
            assert_eq!(parse(json), CellInput::Typed(Value::Error("#N/A".into())));
        }
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
