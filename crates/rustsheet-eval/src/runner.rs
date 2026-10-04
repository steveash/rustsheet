//! Executes cases against an [`Engine`] and scores every checked cell.

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::case::{
    canonical_key, cell_key, split_qualified, CaseSpec, CellInput, OracleFile, CASE_FORMAT,
};
use crate::cellref::Addr;
use crate::engine::{Engine, Evaluated};
use crate::suite::{CaseFile, Suite, SuiteKind};
use crate::value::{compare, Value, Verdict};
use crate::volatile::{pinned_clock_serial, scorable_cells, volatile_cells, FormulaCell};
use crate::xlsx_snapshot::read_snapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    Pass,
    Fail,
    /// No expectations available yet (needs an oracle run).
    Pending,
    /// Engine returned an error while loading/evaluating.
    EngineError,
    Panic,
    Timeout,
    /// The case file itself is unreadable or malformed.
    InvalidCase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellResult {
    pub sheet: String,
    pub cell: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    pub expected: Value,
    pub actual: Value,
    pub verdict: Verdict,
}

impl CellResult {
    /// Stable identifier used in baselines: `Sheet!A1`.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}!{}", self.sheet, self.cell)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub id: String,
    pub suite: String,
    pub status: CaseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub expected_source: String,
    pub checked: usize,
    pub passed: usize,
    pub skipped_volatile: usize,
    pub millis: u64,
    /// Every scored cell (reports may drop passing cells when serializing).
    pub cells: Vec<CellResult>,
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub timeout: Duration,
    /// Substring filter on case ids.
    pub filter: Option<String>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            filter: None,
        }
    }
}

/// Resolves the oracle value for (sheet, cell).
type Lookup = Box<dyn Fn(&str, Addr) -> Option<Value>>;

/// One expectation: a cell to score.
struct Expectation {
    sheet: String,
    addr: Addr,
    formula: Option<String>,
    expected: Value,
}

enum Prepared {
    Xlsx { bytes: Vec<u8> },
    Spec { spec: Box<CaseSpec> },
}

struct PreparedCase {
    input: Prepared,
    expectations: Vec<Expectation>,
    expected_source: String,
    skipped_volatile: usize,
}

/// Runs all cases of the given suites in parallel.
#[must_use]
pub fn run_suites(
    engine: &Arc<dyn Engine + Send>,
    suites: &[Suite],
    opts: &RunOptions,
) -> Vec<CaseResult> {
    let mut jobs = Vec::new();
    for suite in suites {
        match suite.cases() {
            Ok(cases) => jobs.extend(cases.into_iter().map(|c| (suite.clone(), c))),
            Err(e) => jobs.push((
                suite.clone(),
                CaseFile {
                    id: format!("{}/<discovery>", suite.manifest.name),
                    path: suite.root().join(format!("<error: {e}>")),
                    sidecar: PathBuf::new(),
                },
            )),
        }
    }
    if let Some(f) = &opts.filter {
        jobs.retain(|(_, c)| c.id.contains(f.as_str()));
    }
    jobs.par_iter()
        .map(|(suite, case)| run_case(engine, suite, case, opts.timeout))
        .collect()
}

fn invalid(suite: &Suite, case: &CaseFile, message: String) -> CaseResult {
    CaseResult {
        id: case.id.clone(),
        suite: suite.manifest.name.clone(),
        status: CaseStatus::InvalidCase,
        message: Some(message),
        expected_source: String::new(),
        checked: 0,
        passed: 0,
        skipped_volatile: 0,
        millis: 0,
        cells: Vec::new(),
    }
}

/// Runs a single case with panic isolation and a timeout.
#[must_use]
pub fn run_case(
    engine: &Arc<dyn Engine + Send>,
    suite: &Suite,
    case: &CaseFile,
    timeout: Duration,
) -> CaseResult {
    let prepared = match prepare(suite, case) {
        Ok(p) => p,
        Err(e) => return invalid(suite, case, e),
    };
    let mut result = CaseResult {
        id: case.id.clone(),
        suite: suite.manifest.name.clone(),
        status: CaseStatus::Pending,
        message: None,
        expected_source: prepared.expected_source.clone(),
        checked: 0,
        passed: 0,
        skipped_volatile: prepared.skipped_volatile,
        millis: 0,
        cells: Vec::new(),
    };
    // Cases without expectations still run: a panic, timeout or load error is a crash we can
    // catch before any oracle value exists (T-0007).
    let (tx, rx) = mpsc::channel();
    let engine = Arc::clone(engine);
    let start = Instant::now();
    let spawned = std::thread::Builder::new()
        .name(format!("case:{}", case.id))
        .stack_size(256 * 1024 * 1024) // deep dependency chains recurse in the evaluator
        .spawn(move || {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let evaluated = match &prepared.input {
                    Prepared::Xlsx { bytes } => engine.evaluate_xlsx(bytes),
                    Prepared::Spec { spec } => engine.evaluate_spec(spec),
                }?;
                Ok::<_, String>(score(evaluated.as_ref(), &prepared.expectations))
            }));
            let _ = tx.send(outcome);
        });
    if let Err(e) = spawned {
        result.status = CaseStatus::EngineError;
        result.message = Some(format!("could not spawn worker: {e}"));
        return result;
    }
    let received = rx.recv_timeout(timeout);
    result.millis = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    match received {
        Err(_) => {
            result.status = CaseStatus::Timeout;
            result.message = Some(format!("exceeded {}s", timeout.as_secs()));
        }
        Ok(Err(panic)) => {
            result.status = CaseStatus::Panic;
            result.message = Some(panic_message(panic.as_ref()));
        }
        Ok(Ok(Err(e))) => {
            result.status = CaseStatus::EngineError;
            result.message = Some(e);
        }
        Ok(Ok(Ok(cells))) => {
            result.checked = cells.len();
            result.passed = cells.iter().filter(|c| c.verdict.is_pass()).count();
            result.status = if result.checked == 0 {
                CaseStatus::Pending
            } else if result.passed == result.checked {
                CaseStatus::Pass
            } else {
                CaseStatus::Fail
            };
            result.cells = cells;
        }
    }
    result
}

fn panic_message(p: &(dyn std::any::Any + Send)) -> String {
    p.downcast_ref::<String>()
        .cloned()
        .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_else(|| "panic with non-string payload".into())
}

fn score(evaluated: &dyn Evaluated, expectations: &[Expectation]) -> Vec<CellResult> {
    expectations
        .iter()
        .map(|e| {
            let (actual, verdict) = match evaluated.value(&e.sheet, e.addr) {
                Some(actual) => {
                    let verdict = compare(&e.expected, &actual);
                    (actual, verdict)
                }
                None => (
                    Value::Error("<missing sheet>".into()),
                    Verdict::TypeMismatch,
                ),
            };
            CellResult {
                sheet: e.sheet.clone(),
                cell: e.addr.to_string(),
                formula: e.formula.clone(),
                expected: e.expected.clone(),
                actual,
                verdict,
            }
        })
        .collect()
}

fn load_oracle(case: &CaseFile) -> Result<Option<OracleFile>, String> {
    let path = &case.sidecar;
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn prepare(suite: &Suite, case: &CaseFile) -> Result<PreparedCase, String> {
    match suite.manifest.kind {
        SuiteKind::Xlsx => prepare_xlsx(suite, case),
        SuiteKind::Spec => prepare_spec(case),
    }
}

fn prepare_xlsx(suite: &Suite, case: &CaseFile) -> Result<PreparedCase, String> {
    let bytes = std::fs::read(&case.path).map_err(|e| e.to_string())?;
    let snapshot = read_snapshot(&bytes).map_err(|e| e.to_string())?;
    let oracle = load_oracle(case)?;
    let (source, lookup): (String, Lookup) = if let Some(o) = oracle {
        let kind = o.source.kind.clone();
        (
            kind,
            Box::new(move |sheet: &str, addr: Addr| {
                o.sheets
                    .get(sheet)
                    .and_then(|m| m.get(&addr.to_string()))
                    .cloned()
            }),
        )
    } else if suite.manifest.trusts_cached_values(&snapshot) {
        let app = snapshot.application.clone().unwrap_or_default();
        let cached = snapshot.values();
        (
            format!("excel-cached ({app})"),
            Box::new(move |s: &str, a: Addr| cached.get(&(s.to_string(), a)).cloned()),
        )
    } else {
        return Ok(PreparedCase {
            input: Prepared::Xlsx { bytes },
            expectations: Vec::new(),
            expected_source: "none".into(),
            skipped_volatile: 0,
        });
    };
    let (scorable, skipped_volatile) = scorable_cells(
        &snapshot.sheets,
        pinned_clock_serial(&snapshot.sheets).is_some(),
    );
    let expectations = scorable
        .into_iter()
        .filter_map(|(sheet, addr, cell)| {
            Some(Expectation {
                sheet: sheet.to_string(),
                addr,
                formula: cell.formula.clone(),
                expected: lookup(sheet, addr)?,
            })
        })
        .collect();
    Ok(PreparedCase {
        input: Prepared::Xlsx { bytes },
        expectations,
        expected_source: source,
        skipped_volatile,
    })
}

fn prepare_spec(case: &CaseFile) -> Result<PreparedCase, String> {
    let text = std::fs::read_to_string(&case.path).map_err(|e| e.to_string())?;
    let spec: CaseSpec = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if spec.format != CASE_FORMAT {
        return Err(format!("unsupported case format {:?}", spec.format));
    }
    // Hand keys may be written `Sheet1!B1` or `'Sheet1'!B1`; canonicalise them so an oracle value
    // for the same cell *replaces* the hand value (the oracle wins) instead of being scored beside it.
    let mut expected: BTreeMap<String, Value> = BTreeMap::new();
    for (key, value) in &spec.expected {
        let canonical = canonical_key(key).ok_or_else(|| format!("bad cell key {key}"))?;
        if expected.insert(canonical, value.clone()).is_some() {
            return Err(format!("duplicate expectation for cell {key}"));
        }
    }
    let mut source = spec
        .expected_source
        .clone()
        .unwrap_or_else(|| "inline".into());
    if let Some(o) = load_oracle(case)? {
        source.clone_from(&o.source.kind);
        for (sheet, cells) in o.sheets {
            for (a1, v) in cells {
                let addr = Addr::parse(&a1)
                    .ok_or_else(|| format!("bad cell key {sheet}!{a1} in oracle sidecar"))?;
                expected.insert(cell_key(&sheet, addr), v);
            }
        }
    }
    let formula_text = |input: &CellInput| match input {
        CellInput::Str(f) => f.strip_prefix('=').map(str::to_string),
        _ => None,
    };
    let formula_of = |sheet: &str, a1: &str| {
        spec.sheets
            .iter()
            .find(|s| s.name == sheet)
            .and_then(|s| s.cells.get(a1))
            .and_then(formula_text)
    };
    let formulas: Vec<FormulaCell> = spec
        .sheets
        .iter()
        .flat_map(|sheet| {
            sheet.cells.iter().filter_map(|(a1, input)| {
                Some((sheet.name.clone(), Addr::parse(a1)?, formula_text(input)?))
            })
        })
        .collect();
    // Spec workbooks have no METADATA sheet, so the clock is never pinned: NOW/TODAY stay skipped.
    let volatile = volatile_cells(&formulas, false);
    let wanted: Vec<String> = if spec.check.is_empty() {
        expected.keys().cloned().collect()
    } else {
        spec.check.clone()
    };
    let mut expectations = Vec::new();
    let mut skipped_volatile = 0;
    for q in wanted {
        let (sheet, a1) = split_qualified(&q).ok_or_else(|| format!("bad cell key {q}"))?;
        let addr = Addr::parse(&a1).ok_or_else(|| format!("bad cell key {q}"))?;
        let Some(value) = expected.get(&cell_key(&sheet, addr)).cloned() else {
            continue; // pending oracle for this cell
        };
        let formula = formula_of(&sheet, &a1);
        if volatile.contains(&(sheet.clone(), addr)) {
            skipped_volatile += 1;
            continue;
        }
        expectations.push(Expectation {
            sheet,
            addr,
            formula,
            expected: value,
        });
    }
    Ok(PreparedCase {
        input: Prepared::Spec {
            spec: Box::new(spec),
        },
        expectations,
        expected_source: source,
        skipped_volatile,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case::ORACLE_FORMAT;
    use crate::suite::{oracle_sidecar, SuiteManifest};

    /// Engine whose behaviour is chosen by the spec's `description` ("panic", "error", else ok).
    struct Scripted;

    struct NoValues;
    impl Evaluated for NoValues {
        fn value(&self, _: &str, _: Addr) -> Option<Value> {
            None
        }
    }

    impl Engine for Scripted {
        fn name(&self) -> &'static str {
            "scripted"
        }
        fn evaluate_xlsx(&self, _: &[u8]) -> Result<Box<dyn Evaluated>, String> {
            Err("unused".into())
        }
        fn evaluate_spec(&self, spec: &CaseSpec) -> Result<Box<dyn Evaluated>, String> {
            match spec.description.as_str() {
                "panic" => panic!("boom"),
                "error" => Err("cannot materialize".into()),
                _ => Ok(Box::new(NoValues)),
            }
        }
    }

    fn run_pending(description: &str) -> CaseResult {
        let dir =
            std::env::temp_dir().join(format!("rs-runner-{}-{description}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("a.case.json");
        let json = format!(
            r#"{{"format":"{CASE_FORMAT}","description":"{description}","sheets":[{{"name":"S","cells":{{"A1":"=1+1"}}}}]}}"#
        );
        std::fs::write(&path, json).expect("write case");
        let manifest: SuiteManifest =
            toml::from_str("name = \"t\"\nkind = \"spec\"\nlicense = \"MIT\"\n").expect("manifest");
        let suite = Suite {
            manifest,
            dir: dir.clone(),
            cache_dir: PathBuf::new(),
        };
        let case = CaseFile {
            id: "t/a".into(),
            sidecar: oracle_sidecar(&path),
            path,
        };
        let engine: Arc<dyn Engine + Send> = Arc::new(Scripted);
        let r = run_case(&engine, &suite, &case, Duration::from_secs(5));
        let _ = std::fs::remove_dir_all(dir);
        r
    }

    #[test]
    fn pending_case_that_evaluates_cleanly_stays_pending() {
        assert_eq!(run_pending("ok").status, CaseStatus::Pending);
    }

    #[test]
    fn pending_case_that_panics_is_reported_as_panic() {
        let r = run_pending("panic");
        assert_eq!(r.status, CaseStatus::Panic);
        assert_eq!(r.message.as_deref(), Some("boom"));
    }

    /// An oracle sidecar replaces the hand expectation for the same cell, however the hand key
    /// was written; it is never scored next to it (smoke/d-function-criteria J2, 2026-10-04).
    #[test]
    fn oracle_value_replaces_hand_value_for_the_same_cell() {
        let dir =
            std::env::temp_dir().join(format!("rs-runner-{}-oracle-wins", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("a.case.json");
        let case = format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=6","A2":"=7","B1":"=8"}}}},
               {{"name":"My S","cells":{{"A1":"=9"}}}}],
               "expected":{{"S!A1":{{"t":"n","v":6}},"'S'!$A$2":{{"t":"n","v":7}},"S!B1":{{"t":"n","v":8}},
                            "'My S'!A1":{{"t":"n","v":9}}}}}}"#
        );
        std::fs::write(&path, case).expect("write case");
        let oracle = format!(
            r#"{{"format":"{ORACLE_FORMAT}","source":{{"kind":"excel-graph","application":"x"}},
               "sheets":{{"S":{{"A1":{{"t":"n","v":0}},"A2":{{"t":"n","v":1}}}},"My S":{{"A1":{{"t":"n","v":2}}}}}}}}"#
        );
        std::fs::write(dir.join("a.oracle.json"), oracle).expect("write oracle");
        let prepared = prepare_spec(&CaseFile {
            id: "t/a".into(),
            sidecar: oracle_sidecar(&path),
            path,
        });
        let _ = std::fs::remove_dir_all(&dir);
        let prepared = prepared.expect("prepare");
        let mut got: Vec<(String, String, Value)> = prepared
            .expectations
            .into_iter()
            .map(|e| (e.sheet, e.addr.to_string(), e.expected))
            .collect();
        got.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        assert_eq!(
            got,
            vec![
                ("My S".into(), "A1".into(), Value::Number(2.0)),
                ("S".into(), "A1".into(), Value::Number(0.0)),
                ("S".into(), "A2".into(), Value::Number(1.0)),
                ("S".into(), "B1".into(), Value::Number(8.0)), // no oracle value: hand stays
            ]
        );
        assert_eq!(prepared.expected_source, "excel-graph");
    }

    #[test]
    fn duplicate_hand_keys_and_bad_oracle_keys_are_errors() {
        let dir = std::env::temp_dir().join(format!("rs-runner-{}-bad-keys", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("a.case.json");
        let case_with = |expected: &str| {
            format!(
                r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=1"}}}}],"expected":{{{expected}}}}}"#
            )
        };
        let file = CaseFile {
            id: "t/a".into(),
            path: path.clone(),
            sidecar: oracle_sidecar(&path),
        };
        std::fs::write(
            &path,
            case_with(r#""S!A1":{"t":"n","v":1},"'S'!$A$1":{"t":"n","v":2}"#),
        )
        .expect("write case");
        let duplicate = prepare_spec(&file).err();
        std::fs::write(&path, case_with(r#""S!A1":{"t":"n","v":1}"#)).expect("write case");
        let oracle = format!(
            r#"{{"format":"{ORACLE_FORMAT}","source":{{"kind":"k","application":"x"}},"sheets":{{"S":{{"nope":{{"t":"n","v":1}}}}}}}}"#
        );
        std::fs::write(dir.join("a.oracle.json"), oracle).expect("write oracle");
        let bad_oracle = prepare_spec(&file).err();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(duplicate.is_some_and(|e| e.contains("duplicate expectation")));
        assert!(bad_oracle.is_some_and(|e| e.contains("in oracle sidecar")));
    }

    #[test]
    fn pending_case_with_engine_error_is_reported() {
        let r = run_pending("error");
        assert_eq!(r.status, CaseStatus::EngineError);
    }
}
