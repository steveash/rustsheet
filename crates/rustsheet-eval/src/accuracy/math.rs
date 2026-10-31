//! The `rustsheet-math/v1` sidecar (SPEC-0029 "Point fields"): a case's 50-digit math values,
//! written by `rustsheet-accuracy` in tools/ (T-3357) next to the case's oracle sidecar.
//!
//! ```json
//! {"format":"rustsheet-math/v1",
//!  "source":{"generator":"rustsheet-accuracy","version":"1","dps":60,"seed":1,"input_sha256":"…"},
//!  "sheets":{"Calc":{"A1":{"t":"n","v":"-40.26…e0","f":"=T.INV(1.87E-297,3937)","s":"extreme"},
//!                    "A2":{"t":"e","why":"p outside (0,1)","f":"=T.INV(0,3)","s":"edge"}}}}
//! ```
//!
//! The 50 digits are parsed to the nearest double (`str::parse`, correctly rounded): ε = 1e-9 is
//! seven orders of magnitude above that rounding, so no big-decimal comparison is needed. A value
//! whose `f` differs from the cell's formula is stale and never attaches.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::envelope::MathValue;
use crate::cellref::Addr;

/// The `format` of a math sidecar.
pub const MATH_FORMAT: &str = "rustsheet-math/v1";

/// The stratum a generated point was drawn from (SPEC-0029 "Strata"); absent on eval cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stratum {
    Typical,
    Tail,
    Edge,
    Extreme,
    LogUniform,
}

impl Stratum {
    /// The sidecar's name for the stratum.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Typical => "typical",
            Self::Tail => "tail",
            Self::Edge => "edge",
            Self::Extreme => "extreme",
            Self::LogUniform => "log-uniform",
        }
    }
}

/// Provenance of a math sidecar. Only `generator` is required; the rest documents the run.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MathSource {
    pub generator: String,
    #[serde(default)]
    pub version: Option<serde_json::Value>,
    #[serde(default)]
    pub dps: Option<u32>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub input_sha256: Option<String>,
}

/// One cell of a math sidecar as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCell {
    t: String,
    #[serde(default)]
    v: Option<String>,
    #[serde(default)]
    why: Option<String>,
    f: String,
    #[serde(default)]
    s: Option<Stratum>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    format: String,
    source: MathSource,
    sheets: BTreeMap<String, BTreeMap<String, RawCell>>,
}

/// A cell's math reference.
#[derive(Debug, Clone, PartialEq)]
pub struct MathCell {
    pub value: MathValue,
    /// The formula the value was computed for, without a leading `=`.
    pub formula: String,
    pub stratum: Option<Stratum>,
}

/// What a math sidecar holds for one cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Attached<'a> {
    /// A math value for the cell's formula.
    Value(&'a MathCell),
    /// A math value for another formula: dropped and reported (SPEC-0029).
    Stale(&'a MathCell),
    /// No math value for the cell.
    Missing,
}

/// Why a math sidecar is unusable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MathError {
    /// The file cannot be read or is not JSON of the sidecar's shape.
    #[error("{path}: {message}")]
    Unreadable { path: String, message: String },
    /// The file reads, but a field breaks the format.
    #[error("{path}: {why}")]
    Format { path: String, why: String },
}

impl MathError {
    /// The same error, naming the file it is about.
    fn at(self, path: String) -> Self {
        match self {
            Self::Unreadable { message, .. } => Self::Unreadable { path, message },
            Self::Format { why, .. } => Self::Format { path, why },
        }
    }
}

/// A parsed math sidecar.
#[derive(Debug, Clone, PartialEq)]
pub struct MathTable {
    pub source: MathSource,
    /// Sheet → canonical A1 → cell.
    pub cells: BTreeMap<String, BTreeMap<String, MathCell>>,
}

/// The math sidecar beside an oracle sidecar path: `foo.oracle.json` → `foo.math.json`, so a
/// suite's `oracle_dir` mirroring applies to both (SPEC-0029 "Math references for eval cells").
/// The writer, `math_path` in tools/src/rustsheet_tools/accuracy/eval_math.py, must agree.
#[must_use]
pub fn math_sidecar(oracle_sidecar: &Path) -> PathBuf {
    let name = oracle_sidecar
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let stem = name.strip_suffix(".oracle.json").unwrap_or(&name);
    oracle_sidecar.with_file_name(format!("{stem}.math.json"))
}

impl MathTable {
    /// Reads the sidecar at `path`; `None` when there is none.
    ///
    /// # Errors
    /// When the file is unreadable or breaks the format.
    pub fn load(path: &Path) -> Result<Option<MathTable>, MathError> {
        if !path.exists() {
            return Ok(None);
        }
        let shown = path.display().to_string();
        let text = std::fs::read_to_string(path).map_err(|e| MathError::Unreadable {
            path: shown.clone(),
            message: e.to_string(),
        })?;
        Self::parse(&text).map(Some).map_err(|e| e.at(shown))
    }

    /// Parses a sidecar's text.
    ///
    /// # Errors
    /// When the text is not a well-formed `rustsheet-math/v1` sidecar.
    pub fn parse(text: &str) -> Result<MathTable, MathError> {
        let raw: RawFile = serde_json::from_str(text).map_err(|e| MathError::Unreadable {
            path: String::new(),
            message: e.to_string(),
        })?;
        let bad = |why: String| MathError::Format {
            path: String::new(),
            why,
        };
        if raw.format != MATH_FORMAT {
            return Err(bad(format!("unsupported format {:?}", raw.format)));
        }
        let mut cells = BTreeMap::new();
        for (sheet, sheet_cells) in raw.sheets {
            let mut out = BTreeMap::new();
            for (a1, cell) in sheet_cells {
                let addr = Addr::parse(&a1).ok_or_else(|| bad(format!("bad cell {sheet}!{a1}")))?;
                let value = match (cell.t.as_str(), cell.v, cell.why) {
                    ("n", Some(v), None) => MathValue::Number(
                        v.parse::<f64>()
                            .ok()
                            .filter(|x| x.is_finite())
                            .ok_or_else(|| bad(format!("{sheet}!{a1}: bad math value {v:?}")))?,
                    ),
                    ("e", None, Some(why)) => MathValue::DomainError(why),
                    _ => {
                        return Err(bad(format!(
                            "{sheet}!{a1}: a cell is {{\"t\":\"n\",\"v\":<digits>}} or {{\"t\":\"e\",\"why\":<text>}}"
                        )))
                    }
                };
                let formula = strip_equals(&cell.f).to_string();
                let cell = MathCell {
                    value,
                    formula,
                    stratum: cell.s,
                };
                if out.insert(addr.to_string(), cell).is_some() {
                    return Err(bad(format!("duplicate cell {sheet}!{a1}")));
                }
            }
            cells.insert(sheet, out);
        }
        Ok(MathTable {
            source: raw.source,
            cells,
        })
    }

    /// The math reference for `sheet!addr`, whose formula (without `=`) is `formula`.
    #[must_use]
    pub fn attach(&self, sheet: &str, addr: Addr, formula: Option<&str>) -> Attached<'_> {
        let Some(cell) = self.cells.get(sheet).and_then(|m| m.get(&addr.to_string())) else {
            return Attached::Missing;
        };
        if formula.map(strip_equals) == Some(cell.formula.as_str()) {
            Attached::Value(cell)
        } else {
            Attached::Stale(cell)
        }
    }
}

/// Formula text without its leading `=`: the sidecar may write either form.
fn strip_equals(f: &str) -> &str {
    f.strip_prefix('=').unwrap_or(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIDECAR: &str = r#"{"format":"rustsheet-math/v1",
        "source":{"generator":"rustsheet-accuracy","version":"0.1","dps":60,"seed":7,"input_sha256":"ab"},
        "sheets":{"Calc":{
          "A1":{"t":"n","v":"-40.260321845318434217930963425227263164418587702213e0","f":"=T.INV(1.87E-297,3937)","s":"extreme"},
          "$A$2":{"t":"e","why":"p outside (0,1)","f":"T.INV(0,3)","s":"edge"}}}}"#;

    fn at(a1: &str) -> Addr {
        Addr::parse(a1).expect("addr")
    }

    #[test]
    fn parses_numbers_domain_errors_and_strata() {
        let t = MathTable::parse(SIDECAR).expect("parses");
        let Attached::Value(a1) = t.attach("Calc", at("A1"), Some("T.INV(1.87E-297,3937)")) else {
            panic!("A1 attaches");
        };
        let nearest: f64 = "-40.260321845318434217930963425227263164418587702213"
            .parse()
            .expect("digits");
        assert_eq!(a1.value, MathValue::Number(nearest));
        assert!((nearest + 40.260_321_845_318_43).abs() < 1e-13);
        assert_eq!(a1.stratum, Some(Stratum::Extreme));
        let Attached::Value(a2) = t.attach("Calc", at("A2"), Some("=T.INV(0,3)")) else {
            panic!("A2 attaches under its canonical key");
        };
        assert_eq!(a2.value, MathValue::DomainError("p outside (0,1)".into()));
    }

    #[test]
    fn a_value_for_another_formula_is_stale() {
        let t = MathTable::parse(SIDECAR).expect("parses");
        assert!(matches!(
            t.attach("Calc", at("A1"), Some("T.INV(1.87E-297,3938)")),
            Attached::Stale(_)
        ));
        assert!(matches!(
            t.attach("Calc", at("A1"), None),
            Attached::Stale(_)
        ));
        assert_eq!(t.attach("Calc", at("A3"), Some("X()")), Attached::Missing);
        assert_eq!(t.attach("Other", at("A1"), Some("X()")), Attached::Missing);
    }

    #[test]
    fn malformed_sidecars_are_refused() {
        let wrap = |cell: &str| {
            format!(
                r#"{{"format":"rustsheet-math/v1","source":{{"generator":"g"}},"sheets":{{"Calc":{{"A1":{cell}}}}}}}"#
            )
        };
        assert!(MathTable::parse(&wrap(r#"{"t":"n","v":"1.5","f":"X()"}"#)).is_ok());
        for bad in [
            r#"{"t":"n","v":"inf","f":"X()"}"#,
            r#"{"t":"n","v":"1e400","f":"X()"}"#,
            r#"{"t":"n","v":"abc","f":"X()"}"#,
            r#"{"t":"n","f":"X()"}"#,
            r##"{"t":"e","v":"#NUM!","f":"X()"}"##,
            r#"{"t":"n","v":"1","f":"X()","s":"weird"}"#,
            r#"{"t":"n","v":"1"}"#,
        ] {
            assert!(MathTable::parse(&wrap(bad)).is_err(), "{bad}");
        }
        assert!(MathTable::parse(&SIDECAR.replace("math/v1", "math/v2")).is_err());
    }

    #[test]
    fn the_math_sidecar_sits_beside_the_oracle_sidecar() {
        assert_eq!(
            math_sidecar(Path::new("t/T.INV/s1.oracle.json")),
            PathBuf::from("t/T.INV/s1.math.json")
        );
    }
}
