//! Executes cases against an [`Engine`] and scores every checked cell.

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::case::{split_qualified, CaseSpec, CellInput, OracleFile, CASE_FORMAT};
use crate::cellref::Addr;
use crate::engine::{Engine, Evaluated};
use crate::suite::{oracle_sidecar, CaseFile, Suite, SuiteKind};
use crate::value::{compare, Value, Verdict};
use crate::xlsx_snapshot::{read_snapshot, WorkbookSnapshot};

/// Functions whose results legitimately differ between runs; such cells are not scored.
static VOLATILE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:_xlfn\.)?(NOW|TODAY|RAND|RANDBETWEEN|RANDARRAY)\s*\(")
        .unwrap_or_else(|e| unreachable!("static regex: {e}"))
});

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
    let path = oracle_sidecar(&case.path);
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn is_volatile(formula: Option<&str>) -> bool {
    formula.is_some_and(|f| VOLATILE.is_match(f))
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
    } else if suite.manifest.use_cached_values
        && (!suite.manifest.require_excel_authored || snapshot.is_excel_authored())
    {
        let app = snapshot.application.clone().unwrap_or_default();
        let cached = cached_lookup(&snapshot);
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
    let mut expectations = Vec::new();
    let mut skipped_volatile = 0;
    for sheet in &snapshot.sheets {
        for (addr, cell) in &sheet.cells {
            if !cell.is_formula() {
                continue;
            }
            if is_volatile(cell.formula.as_deref()) {
                skipped_volatile += 1;
                continue;
            }
            if let Some(expected) = lookup(&sheet.name, *addr) {
                expectations.push(Expectation {
                    sheet: sheet.name.clone(),
                    addr: *addr,
                    formula: cell.formula.clone(),
                    expected,
                });
            }
        }
    }
    Ok(PreparedCase {
        input: Prepared::Xlsx { bytes },
        expectations,
        expected_source: source,
        skipped_volatile,
    })
}

fn cached_lookup(snapshot: &WorkbookSnapshot) -> BTreeMap<(String, Addr), Value> {
    let mut map = BTreeMap::new();
    for sheet in &snapshot.sheets {
        for (addr, cell) in &sheet.cells {
            map.insert((sheet.name.clone(), *addr), cell.value.clone());
        }
    }
    map
}

fn prepare_spec(case: &CaseFile) -> Result<PreparedCase, String> {
    let text = std::fs::read_to_string(&case.path).map_err(|e| e.to_string())?;
    let spec: CaseSpec = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    if spec.format != CASE_FORMAT {
        return Err(format!("unsupported case format {:?}", spec.format));
    }
    let mut expected = spec.expected.clone();
    let mut source = spec
        .expected_source
        .clone()
        .unwrap_or_else(|| "inline".into());
    if let Some(o) = load_oracle(case)? {
        source.clone_from(&o.source.kind);
        for (sheet, cells) in o.sheets {
            for (a1, v) in cells {
                expected.insert(format!("'{}'!{a1}", sheet.replace('\'', "''")), v);
            }
        }
    }
    let formula_of = |sheet: &str, a1: &str| {
        spec.sheets
            .iter()
            .find(|s| s.name == sheet)
            .and_then(|s| s.cells.get(a1))
            .and_then(|c| match c {
                CellInput::Str(f) if f.starts_with('=') => Some(f[1..].to_string()),
                _ => None,
            })
    };
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
        let normalized = format!("'{}'!{addr}", sheet.replace('\'', "''"));
        let Some(value) = expected
            .get(&q)
            .or_else(|| expected.get(&normalized))
            .cloned()
        else {
            continue; // pending oracle for this cell
        };
        let formula = formula_of(&sheet, &a1);
        if is_volatile(formula.as_deref()) {
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
    use crate::suite::SuiteManifest;
    use std::path::PathBuf;

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

    #[test]
    fn pending_case_with_engine_error_is_reported() {
        let r = run_pending("error");
        assert_eq!(r.status, CaseStatus::EngineError);
    }
}
