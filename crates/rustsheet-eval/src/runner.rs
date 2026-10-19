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
    /// An xlsx case's workbook-scope defined names, upper-case name → formula text without `=`;
    /// the milestone function scope follows the names a formula calls (T-1970).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub defined_names: BTreeMap<String, String>,
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
    defined_names: BTreeMap<String, String>,
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
        defined_names: BTreeMap::new(),
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
        defined_names: prepared.defined_names.clone(),
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
            defined_names: BTreeMap::new(),
        });
    };
    let (scorable, skipped_volatile) =
        scorable_cells(&snapshot, pinned_clock_serial(&snapshot.sheets).is_some());
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
    let defined_names = snapshot
        .defined_names
        .iter()
        .filter(|n| n.sheet.is_none())
        .map(|n| (n.name.to_ascii_uppercase(), n.body.clone()))
        .collect();
    Ok(PreparedCase {
        input: Prepared::Xlsx { bytes },
        expectations,
        extent: None,
        expected_source: source,
        skipped_volatile,
        tags: Vec::new(),
        text: Vec::new(),
        defined_names,
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
        defined_names: BTreeMap::new(),
        input: Prepared::Spec {
            spec: Box::new(spec),
        },
        expectations,
        expected_source: source,
        skipped_volatile,
    })
}

#[cfg(test)]
mod tests;
