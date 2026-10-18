//! Executes cases against an [`Engine`] and scores every checked cell.

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::case::{
    canonical_key, cell_key, is_excel_evidence, load_oracle, split_qualified, CaseSpec, CellInput,
    CASE_FORMAT, GRAPH_ORACLE,
};
use crate::case_log;
use crate::cellref::Addr;
use crate::deviations::{self, Outcome};
use crate::engine::{Engine, Evaluated};
use crate::overrides;
use crate::spill_extent::SpillExtent;
use crate::suite::{CaseFile, Suite, SuiteKind};
use crate::text_expect::{self, TextExpectation, TextResult};
use crate::value::{compare, Value, Verdict};
use crate::volatile::{pinned_clock_serial, scorable_cells, spec_volatile_cells, FormulaCell};
use crate::xlsx_snapshot::{is_undecoded, read_snapshot};

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
    /// For an array-member cell (no formula of its own), the formula of its array anchor, so the
    /// milestone function scope can classify it (T-1877).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_formula: Option<String>,
    pub expected: Value,
    pub actual: Value,
    pub verdict: Verdict,
    /// Whether the expected value came from Excel (oracle sidecar or Excel-authored cached value),
    /// as opposed to a hand-written one. The evidence report counts only these cells (T-0044).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub from_excel: bool,
    /// Whether the suite's `deviations` file lists the cell (decision 0036). A listed cell that
    /// passes is a stale line.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub listed_deviation: bool,
    /// Whether decision 0042 scored the cell a pass (an excel-graph 0 against a kept subnormal).
    /// Such a cell always has a FALSE `(<formula>)=0` companion in its case (`from_graph`);
    /// the report lists these cells so the evidence stays visible (T-1142).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub graph_subnormal: bool,
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
    /// Checked cells that gave their pinned by-design value (decision 0036).
    #[serde(default)]
    pub deviations: usize,
    pub skipped_volatile: usize,
    pub millis: u64,
    /// The spec case's `tags` (none for xlsx cases or unreadable ones); the probe milestone
    /// report reads a case's milestone tag from them (decision 0051).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Every scored cell (reports may drop passing cells when serializing).
    pub cells: Vec<CellResult>,
    /// Displayed-text verdicts, beside the values' (T-1610, decision 0068).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text: Vec<TextResult>,
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
    anchor_formula: Option<String>,
    expected: Value,
    from_excel: bool,
    /// The pinned by-design value when the suite's `deviations` file lists the cell.
    pinned: Option<Value>,
    /// Whether the expectation is an excel-graph value with an excel-graph companion
    /// `=(<formula>)=0` that is FALSE: Excel kept a non-zero result that Graph may report as 0
    /// (`graph_hides_subnormal`).
    from_graph: bool,
}

/// Whether an excel-graph expectation of 0 hides `actual`. The Graph API reports a subnormal
/// cell value as 0.0, so its 0 cannot tell a true 0 from a kept subnormal: Excel keeps
/// `3E-308-2.5E-308` yet Graph reports 0 (probes/subnormal-function-results A69). Only cells
/// whose companion `(x)=0` is FALSE qualify (`from_graph`), so the evidence is Excel's.
/// Decision 0042.
fn graph_hides_subnormal(e: &Expectation, actual: &Value) -> bool {
    e.from_graph
        && e.expected == Value::Number(0.0)
        && matches!(actual, Value::Number(a) if *a != 0.0 && a.abs() < f64::MIN_POSITIVE)
}

enum Prepared {
    Xlsx { bytes: Vec<u8> },
    Spec { spec: Box<CaseSpec> },
}

struct PreparedCase {
    input: Prepared,
    expectations: Vec<Expectation>,
    extent: Option<SpillExtent>,
    expected_source: String,
    skipped_volatile: usize,
    tags: Vec<String>,
    text: Vec<TextExpectation>,
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
        deviations: 0,
        skipped_volatile: 0,
        millis: 0,
        tags: Vec::new(),
        cells: Vec::new(),
        text: Vec::new(),
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
        deviations: 0,
        skipped_volatile: prepared.skipped_volatile,
        millis: 0,
        tags: prepared.tags.clone(),
        cells: Vec::new(),
        text: Vec::new(),
    };
    // Cases without expectations still run: a panic, timeout or load error is a crash we can
    // catch before any oracle value exists (T-0007).
    let (tx, rx) = mpsc::channel();
    let engine = Arc::clone(engine);
    case_log::record("start", &case.id);
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
                let mut cells = score(evaluated.as_ref(), &prepared.expectations);
                let scored = cells.len();
                let extent = prepared.extent.iter();
                cells.extend(extent.flat_map(|e| e.unexpected_cells(evaluated.as_ref())));
                let text = text_expect::score(evaluated.as_ref(), &prepared.text);
                Ok::<_, String>((cells, scored, text))
            }));
            let _ = tx.send(outcome);
        });
    if let Err(e) = spawned {
        case_log::record("end", &case.id);
        result.status = CaseStatus::EngineError;
        result.message = Some(format!("could not spawn worker: {e}"));
        return result;
    }
    let received = rx.recv_timeout(timeout);
    // A timed-out worker keeps running, so it is logged as such rather than as ended.
    let event = if received.is_err() { "timeout" } else { "end" };
    case_log::record(event, &case.id);
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
        Ok(Ok(Ok((cells, scored, text)))) => {
            result.text = text;
            // Spill-extent cells (after `scored`) all fail but stay out of `checked` (decision 0063).
            result.checked = scored;
            result.passed = cells.iter().filter(|c| c.verdict.is_pass()).count();
            result.deviations = cells
                .iter()
                .filter(|c| c.verdict == Verdict::Deviation)
                .count();
            result.status = if result.checked == 0 {
                CaseStatus::Pending
            } else if result.passed + result.deviations == cells.len() {
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
            let mut graph_subnormal = false;
            let (actual, mut verdict) = match evaluated.value(&e.sheet, e.addr) {
                Some(actual) if graph_hides_subnormal(e, &actual) => {
                    graph_subnormal = true;
                    (actual, Verdict::Pass)
                }
                Some(actual) => {
                    let verdict = compare(&e.expected, &actual);
                    (actual, verdict)
                }
                None => (
                    Value::Error("<missing sheet>".into()),
                    Verdict::TypeMismatch,
                ),
            };
            if let Some(pinned) = &e.pinned {
                if deviations::classify(pinned, &e.expected, &actual) == Outcome::Deviation {
                    verdict = Verdict::Deviation;
                }
            }
            CellResult {
                sheet: e.sheet.clone(),
                cell: e.addr.to_string(),
                formula: e.formula.clone(),
                anchor_formula: e.anchor_formula.clone(),
                expected: e.expected.clone(),
                actual,
                verdict,
                from_excel: e.from_excel,
                listed_deviation: e.pinned.is_some(),
                graph_subnormal,
            }
        })
        .collect()
}

fn prepare(suite: &Suite, case: &CaseFile) -> Result<PreparedCase, String> {
    let mut prepared = match suite.manifest.kind {
        SuiteKind::Xlsx => prepare_xlsx(suite, case),
        SuiteKind::Spec => prepare_spec(case),
    }?;
    pin_deviations(suite, case, &mut prepared.expectations)?;
    Ok(prepared)
}

/// The case's id relative to its suite root, as `cached_overrides` and `deviations` lines name it.
fn relative_case_id<'a>(suite: &Suite, case: &'a CaseFile) -> &'a str {
    let prefix = format!("{}/", suite.manifest.name);
    case.id.strip_prefix(&prefix).unwrap_or(&case.id)
}

/// Pins the by-design value of every cell the suite's `deviations` file lists for this case
/// (decision 0036). A line naming a cell that is not scored is an error, as for overrides.
fn pin_deviations(
    suite: &Suite,
    case: &CaseFile,
    expectations: &mut [Expectation],
) -> Result<(), String> {
    let rel = relative_case_id(suite, case);
    for d in deviations::load(suite)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|d| d.case == rel)
    {
        let target = expectations
            .iter_mut()
            .find(|e| e.sheet == d.sheet && e.addr == d.addr)
            .ok_or_else(|| format!("deviation for unscored cell {}!{}", d.sheet, d.addr))?;
        target.pinned = Some(d.pinned);
    }
    Ok(())
}

fn prepare_xlsx(suite: &Suite, case: &CaseFile) -> Result<PreparedCase, String> {
    let bytes = std::fs::read(&case.path).map_err(|e| e.to_string())?;
    let snapshot = read_snapshot(&bytes).map_err(|e| e.to_string())?;
    let oracle = load_oracle(case)?;
    let (source, from_excel, lookup): (String, bool, Lookup) = if let Some(o) = oracle {
        let kind = o.source.kind.clone();
        (
            kind.clone(),
            is_excel_evidence(&kind),
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
            // A suite may trust cached values of any application; only Excel's count as evidence.
            snapshot.is_excel_authored(),
            Box::new(move |s: &str, a: Addr| cached.get(&(s.to_string(), a)).cloned()),
        )
    } else {
        return Ok(PreparedCase {
            input: Prepared::Xlsx { bytes },
            expectations: Vec::new(),
            extent: None,
            expected_source: "none".into(),
            skipped_volatile: 0,
            tags: Vec::new(),
            text: Vec::new(),
        });
    };
    let (scorable, skipped_volatile) = scorable_cells(
        &snapshot.sheets,
        &snapshot.defined_names,
        &snapshot.tables,
        pinned_clock_serial(&snapshot.sheets).is_some(),
    );
    let expectations = scorable
        .into_iter()
        .filter_map(|(sheet, addr, cell)| {
            Some(Expectation {
                sheet: sheet.to_string(),
                addr,
                formula: cell.formula.clone(),
                anchor_formula: cell.anchor_formula(&snapshot.sheets, sheet),
                expected: lookup(sheet, addr).filter(|v| !is_undecoded(v))?,
                from_excel,
                pinned: None,
                from_graph: false,
            })
        })
        .collect();
    let expectations = apply_cached_overrides(suite, case, expectations)?;
    Ok(PreparedCase {
        input: Prepared::Xlsx { bytes },
        expectations,
        extent: None,
        expected_source: source,
        skipped_volatile,
        tags: Vec::new(),
        text: Vec::new(),
    })
}

/// Replaces the expectations a suite's `cached_overrides` file names for this case with their
/// golden values (decision 0032). An override naming a cell that is not scored is an error, so a
/// stale line cannot linger unnoticed.
fn apply_cached_overrides(
    suite: &Suite,
    case: &CaseFile,
    mut expectations: Vec<Expectation>,
) -> Result<Vec<Expectation>, String> {
    let Some(file) = &suite.manifest.cached_overrides else {
        return Ok(expectations);
    };
    let path = suite.dir.join(file);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let suites_dir = suite.dir.parent().unwrap_or(&suite.dir);
    let rel = relative_case_id(suite, case);
    for o in overrides::parse(&text)?.iter().filter(|o| o.case == rel) {
        let expected = overrides::evidence_value(o, suites_dir)?;
        let target = expectations
            .iter_mut()
            .find(|e| e.sheet == o.sheet && e.addr == o.addr)
            .ok_or_else(|| format!("cached override for unscored cell {}!{}", o.sheet, o.addr))?;
        target.expected = expected;
    }
    Ok(expectations)
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
    // Cells whose expectation the sidecar supplies; hand cells beside them are not Excel evidence.
    let mut oracle_cells: BTreeSet<String> = BTreeSet::new();
    let mut oracle_is_excel = false;
    if let Some(o) = load_oracle(case)? {
        source.clone_from(&o.source.kind);
        oracle_is_excel = is_excel_evidence(&o.source.kind);
        for (sheet, cells) in o.sheets {
            for (a1, v) in cells {
                let addr = Addr::parse(&a1)
                    .ok_or_else(|| format!("bad cell key {sheet}!{a1} in oracle sidecar"))?;
                let key = cell_key(&sheet, addr);
                oracle_cells.insert(key.clone());
                expected.insert(key, v);
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
    // Spec workbooks have no METADATA sheet, so the clock is never pinned: NOW/TODAY stay skipped
    // unless the case lists the cell as run-invariant.
    let volatile = spec_volatile_cells(&spec.invariant, &formulas)?;
    let wanted: Vec<String> = if spec.check.is_empty() {
        expected.keys().cloned().collect()
    } else {
        spec.check.clone()
    };
    // Formulas `(x)=0` whose excel-graph value is FALSE: Excel kept `x` non-zero (decision 0042).
    let from_graph_oracle = source == GRAPH_ORACLE;
    let kept_nonzero: BTreeSet<String> = formulas
        .iter()
        .filter(|(sheet, addr, _)| {
            let key = cell_key(sheet, *addr);
            oracle_cells.contains(&key) && expected.get(&key) == Some(&Value::Bool(false))
        })
        .map(|(_, _, formula)| formula.clone())
        .collect();
    let mut expectations = Vec::new();
    let mut skipped_volatile = 0;
    for q in wanted {
        let (sheet, a1) = split_qualified(&q).ok_or_else(|| format!("bad cell key {q}"))?;
        let addr = Addr::parse(&a1).ok_or_else(|| format!("bad cell key {q}"))?;
        let key = cell_key(&sheet, addr);
        let Some(value) = expected.get(&key).filter(|v| !is_undecoded(v)).cloned() else {
            continue; // pending oracle for this cell (or an undecoded rich error)
        };
        let formula = formula_of(&sheet, &a1);
        let from_graph = from_graph_oracle
            && formula
                .as_deref()
                .is_some_and(|f| kept_nonzero.contains(&format!("({f})=0")));
        if volatile.contains(&(sheet.clone(), addr)) {
            skipped_volatile += 1;
            continue;
        }
        expectations.push(Expectation {
            sheet,
            addr,
            formula,
            anchor_formula: None,
            expected: value,
            from_excel: oracle_is_excel && oracle_cells.contains(&key),
            pinned: None,
            from_graph,
        });
    }
    Ok(PreparedCase {
        extent: oracle_is_excel
            .then(|| SpillExtent::new(&spec, &expected, &oracle_cells, &formulas)),
        tags: spec.tags.clone(),
        text: text_expect::expectations(&spec, case)?,
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
        fn stored_cells(&self, _: &str) -> Vec<Addr> {
            Vec::new()
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
        fn implemented_functions(&self) -> Vec<String> {
            Vec::new()
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

    /// A `cached_overrides` line replaces exactly the named cached expectation with the golden
    /// value it points at, and naming an unscored cell is an error (decision 0032).
    #[test]
    fn cached_override_replaces_the_named_expectation() {
        let dir = std::env::temp_dir().join(format!("rs-runner-{}-overrides", std::process::id()));
        let suite_dir = dir.join("calc");
        std::fs::create_dir_all(suite_dir.join("cases")).expect("mkdir");
        std::fs::create_dir_all(dir.join("probes")).expect("mkdir");
        std::fs::write(
            dir.join("probes/p.oracle.json"),
            format!(
                r#"{{"format":"{ORACLE_FORMAT}","source":{{"kind":"excel-graph"}},
                "sheets":{{"Sheet1":{{"B1":{{"t":"n","v":30.0}}}}}}}}"#
            ),
        )
        .expect("write oracle");
        let manifest: SuiteManifest = toml::from_str(
            "name = \"calc\"\nkind = \"xlsx\"\nlicense = \"MIT\"\n\
             cached_overrides = \"o.tsv\"\n",
        )
        .expect("manifest");
        let suite = Suite {
            manifest,
            dir: suite_dir.clone(),
            cache_dir: PathBuf::new(),
        };
        let case = CaseFile {
            id: "calc/D/x.xlsx".into(),
            path: suite_dir.join("cases/D/x.xlsx"),
            sidecar: PathBuf::new(),
        };
        let cached = |a1: &str| Expectation {
            sheet: "S".into(),
            addr: Addr::parse(a1).expect("addr"),
            formula: None,
            anchor_formula: None,
            expected: Value::Error("#VALUE!".into()),
            from_excel: true,
            pinned: None,
            from_graph: false,
        };
        let write_overrides = |cell: &str| {
            let line = format!("D/x.xlsx\tS!{cell}\tprobes/p.oracle.json\tSheet1!B1\n");
            std::fs::write(suite_dir.join("o.tsv"), line).expect("write overrides");
        };
        write_overrides("C71");
        let got = apply_cached_overrides(&suite, &case, vec![cached("C70"), cached("C71")])
            .expect("apply");
        assert_eq!(got[0].expected, Value::Error("#VALUE!".into()));
        assert_eq!(got[1].expected, Value::Number(30.0));
        write_overrides("Z9");
        let unscored = apply_cached_overrides(&suite, &case, vec![cached("C70")]);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(unscored.is_err_and(|e| e.contains("unscored cell S!Z9")));
    }

    /// An excel-graph 0 hides a kept subnormal (decision 0042); other sources and values do not.
    #[test]
    fn graph_zero_hides_only_a_subnormal() {
        let e = |expected: f64, from_graph| Expectation {
            sheet: "S".into(),
            addr: Addr::parse("A1").expect("addr"),
            formula: None,
            anchor_formula: None,
            expected: Value::Number(expected),
            from_excel: true,
            pinned: None,
            from_graph,
        };
        let subnormal = Value::Number(5e-309);
        assert!(graph_hides_subnormal(&e(0.0, true), &subnormal));
        assert!(!graph_hides_subnormal(&e(0.0, false), &subnormal));
        assert!(!graph_hides_subnormal(&e(1.0, true), &subnormal));
        assert!(!graph_hides_subnormal(
            &e(0.0, true),
            &Value::Number(1e-300)
        ));
        assert!(!graph_hides_subnormal(&e(0.0, true), &Value::Number(0.0)));
    }

    /// A listed cell is a deviation when it gives its pinned value, a pass when it matches Excel
    /// and a failure otherwise; a deviation leaves the case passing, and a line naming an
    /// unscored cell is an error (decision 0036).
    #[test]
    fn listed_deviations_score_apart_from_bit_exact() {
        let dir = std::env::temp_dir().join(format!("rs-runner-{}-deviations", std::process::id()));
        std::fs::create_dir_all(dir.join("cases")).expect("mkdir");
        let path = dir.join("cases/a.case.json");
        std::fs::write(
            &path,
            format!(
                r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=1","A2":"=2","A3":"=3"}}}}],
                   "expected":{{"S!A1":{{"t":"n","v":9}},"S!A2":{{"t":"n","v":2}},"S!A3":{{"t":"n","v":9}}}}}}"#
            ),
        )
        .expect("write case");
        let manifest: SuiteManifest = toml::from_str(
            "name = \"t\"\nkind = \"spec\"\nlicense = \"MIT\"\nroot = \"cases\"\n\
             deviations = \"deviations\"\n",
        )
        .expect("manifest");
        let suite = Suite {
            manifest,
            dir: dir.clone(),
            cache_dir: PathBuf::new(),
        };
        let case = CaseFile {
            id: "t/a.case.json".into(),
            sidecar: oracle_sidecar(&path),
            path,
        };
        let write = |cells: &[(&str, &str)]| {
            let lines: Vec<String> = cells
                .iter()
                .map(|(cell, pinned)| {
                    format!(
                        "a.case.json\tS!{cell}\tdecision 0014\t\
                         docs/excel-semantics/arrays.md#x\t{pinned}"
                    )
                })
                .collect();
            std::fs::write(dir.join("deviations"), lines.join("\n")).expect("write deviations");
        };
        let engine: Arc<dyn Engine + Send> = Arc::new(crate::engine::RustSheetEngine);
        let run = || run_case(&engine, &suite, &case, Duration::from_secs(60));
        write(&[("A1", "1"), ("A2", "5")]);
        let r = run();
        let verdicts: Vec<(Verdict, bool)> = r
            .cells
            .iter()
            .map(|c| (c.verdict, c.listed_deviation))
            .collect();
        assert_eq!(
            verdicts,
            [
                (Verdict::Deviation, true),
                (Verdict::Pass, true), // stale: A2 matches Excel
                (Verdict::NumericMismatch, false),
            ]
        );
        assert_eq!((r.checked, r.passed, r.deviations), (3, 1, 1));
        assert_eq!(r.status, CaseStatus::Fail);
        write(&[("A1", "1"), ("A3", "5")]);
        let r = run();
        assert_eq!(r.cells[2].verdict, Verdict::NumericMismatch); // 3 is neither 9 nor 5
        write(&[("A1", "1"), ("A3", "3")]);
        assert_eq!(run().status, CaseStatus::Pass);
        write(&[("Z9", "1")]);
        let unscored = run();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(unscored.status, CaseStatus::InvalidCase);
        assert!(unscored
            .message
            .is_some_and(|m| m.contains("unscored cell S!Z9")));
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
        let excel_flags: Vec<(String, String, bool)> = prepared
            .expectations
            .iter()
            .map(|e| (e.sheet.clone(), e.addr.to_string(), e.from_excel))
            .collect();
        let mut got: Vec<(String, String, Value)> = prepared
            .expectations
            .into_iter()
            .map(|e| (e.sheet, e.addr.to_string(), e.expected))
            .collect();
        got.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
        let excel: Vec<(String, String)> = excel_flags
            .into_iter()
            .filter(|(_, _, from_excel)| *from_excel)
            .map(|(sheet, cell, _)| (sheet, cell))
            .collect();
        // The hand value B1 is not Excel evidence although the sidecar covers its neighbours.
        assert_eq!(excel.len(), 3);
        assert!(!excel.contains(&("S".to_string(), "B1".to_string())));
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

    /// A rich error without an evidence-backed mapping (`#UNDECODED!`, `xlsx_snapshot.rs`) is
    /// never scored: the cell stays pending (poi/link-external-workbook-b Sheet0!A1, T-0203).
    #[test]
    fn undecoded_rich_errors_are_not_scored() {
        let dir = std::env::temp_dir().join(format!("rs-runner-{}-undecoded", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("a.case.json");
        let case = format!(
            r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=1","A2":"=2"}}}}]}}"#
        );
        std::fs::write(&path, case).expect("write case");
        let oracle = format!(
            r##"{{"format":"{ORACLE_FORMAT}","source":{{"kind":"excel-graph","application":"x"}},
               "sheets":{{"S":{{"A1":{{"t":"e","v":"#UNDECODED!"}},"A2":{{"t":"n","v":2}}}}}}}}"##
        );
        std::fs::write(dir.join("a.oracle.json"), oracle).expect("write oracle");
        let prepared = prepare_spec(&CaseFile {
            id: "t/a".into(),
            sidecar: oracle_sidecar(&path),
            path,
        });
        let _ = std::fs::remove_dir_all(&dir);
        let cells: Vec<String> = prepared
            .expect("prepare")
            .expectations
            .into_iter()
            .map(|e| e.addr.to_string())
            .collect();
        assert_eq!(cells, vec!["A2".to_string()]);
    }

    /// A spec's `invariant` cells are scored although they call a volatile function; other
    /// volatile cells stay skipped, and listing a cell that is not volatile is an error (T-1283).
    #[test]
    fn invariant_volatile_cells_are_scored() {
        let dir = std::env::temp_dir().join(format!("rs-runner-{}-invariant", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("a.case.json");
        let case_with = |invariant: &str| {
            format!(
                r#"{{"format":"{CASE_FORMAT}","sheets":[{{"name":"S","cells":{{"A1":"=RANDBETWEEN(5,5)","A2":"=RAND()","A3":"=A1+1","A4":"=1"}}}}],
                   "expected":{{"S!A1":{{"t":"n","v":5}},"S!A2":{{"t":"n","v":0.5}},"S!A3":{{"t":"n","v":6}},"S!A4":{{"t":"n","v":1}}}},
                   "invariant":[{invariant}]}}"#
            )
        };
        let file = CaseFile {
            id: "t/a".into(),
            path: path.clone(),
            sidecar: oracle_sidecar(&path),
        };
        std::fs::write(&path, case_with(r#""S!A1""#)).expect("write case");
        let listed = prepare_spec(&file);
        std::fs::write(&path, case_with(r#""S!A4""#)).expect("write case");
        let stale = prepare_spec(&file).err();
        let _ = std::fs::remove_dir_all(&dir);
        let listed = listed.expect("prepare");
        let mut cells: Vec<String> = listed
            .expectations
            .iter()
            .map(|e| e.addr.to_string())
            .collect();
        cells.sort();
        assert_eq!(cells, ["A1", "A3", "A4"], "A3 reads only the listed A1");
        assert_eq!(listed.skipped_volatile, 1);
        assert!(stale.is_some_and(|e| e.contains("not a volatile formula cell")));
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
