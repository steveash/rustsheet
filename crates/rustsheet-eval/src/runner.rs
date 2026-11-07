//! Executes cases against an [`Engine`] and scores every checked cell.

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::accuracy::envelope::MathValue;
use crate::accuracy::math::{math_sidecar, MathTable};
use crate::argument_coverage::{self, CallSeen};
use crate::case::{CaseSpec, OraclePivot};
use crate::case_log;
use crate::cellref::Addr;
use crate::engine::Engine;
use crate::pivot_freshness::{self, PivotFreshness, SavedPivot};
use crate::pivot_scoring::{self, PivotLayout};
use crate::poi_m8_subset::ReferenceFormCells;
use crate::spill_extent::SpillExtent;
use crate::suite::{CaseFile, Suite};
use crate::text_expect::{self, TextExpectation, TextResult};
use crate::value::{compare, Value, Verdict};

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
    /// Excel would not open the engine's export (SPEC-0033 `excel-refused`, T-4177): always a
    /// failure, whatever the engine gives.
    ExcelRefused,
}

impl CaseStatus {
    /// Whether the case did not run to scoring: it failed to load or evaluate, panicked or timed
    /// out. Such a case fails every exit measure that reads its suite or probe row (decisions
    /// 0146 item 2(b) and 0174); a pending case that runs cleanly stays `Pending` and never does.
    #[must_use]
    pub fn did_not_run(self) -> bool {
        matches!(self, Self::EngineError | Self::Panic | Self::Timeout)
    }
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
    /// A scored expectation, or a cell the spill-extent check added (decision 0140).
    #[serde(default, skip_serializing_if = "CellKind::is_scored")]
    pub kind: CellKind,
    /// The cell's 50-digit math value from the case's math sidecar, when one is there for this
    /// cell's formula (SPEC-0029 "Math references for eval cells", T-3380): R1(c) counts a
    /// visibly-off numeric cell inside Gate A's envelope through it. A value written for another
    /// formula is stale and never attaches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub math: Option<MathValue>,
    /// The value the engine deliberately returns here: Excel's, or the 50-digit math where
    /// decision 0188 R3 has it follow the math. Set once per run by
    /// [`crate::r1c::mark_r3_math`], the test behind R1(c)'s [`crate::r1c::R1c::R3Pass`]. The
    /// closeness buckets read it: a [`Target::Math`] cell is `r3_math`, not visibly off
    /// (decision 0310).
    #[serde(default, skip_serializing_if = "Target::is_excel")]
    pub target: Target,
}

/// What a [`CellResult`]'s engine value is meant to match (decision 0310).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    /// Excel's value: every cell but the ones below.
    #[default]
    Excel,
    /// The 50-digit math, which the engine returns where decision 0188 R3 marks the cell and Excel
    /// is more than G = 1e-4 off it ([`crate::r1c::r3_math`]).
    Math,
}

impl Target {
    /// Whether the cell is held to Excel's value (the serialized default).
    #[must_use]
    pub fn is_excel(&self) -> bool {
        *self == Self::Excel
    }
}

/// Where a [`CellResult`] comes from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellKind {
    /// One of the case's expectations, counted in `checked`.
    #[default]
    Scored,
    /// A cell the engine filled and Excel left empty, outside `checked` (decision 0063); the M7
    /// exit counts these (decision 0140).
    SpillExtent,
}

impl CellKind {
    /// Whether the cell is one of the case's expectations (the serialized default).
    #[must_use]
    pub fn is_scored(&self) -> bool {
        *self == Self::Scored
    }
}

impl CellResult {
    /// A scored Excel-evidence cell of sheet `S` expecting 1 and giving 1, for tests.
    #[cfg(test)]
    pub(crate) fn for_test(cell: &str, formula: Option<&str>, verdict: Verdict) -> CellResult {
        CellResult {
            sheet: "S".to_string(),
            cell: cell.to_string(),
            formula: formula.map(str::to_string),
            anchor_formula: None,
            expected: Value::Number(1.0),
            actual: Value::Number(1.0),
            verdict,
            from_excel: true,
            listed_deviation: false,
            graph_subnormal: false,
            kind: CellKind::Scored,
            math: None,
            target: Target::Excel,
        }
    }

    /// Stable identifier used in baselines: `Sheet!A1`.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}!{}", self.sheet, self.cell)
    }
}

/// The `case!Sheet!A1` name report sections use for one cell; `exit_owners.py` compares these
/// strings across sections (decision 0135), so every report builds them here.
#[must_use]
pub fn cell_id(case: &str, key: &str) -> String {
    format!("{case}!{key}")
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
    /// Formula cells left unscored because they call an add-in function (`_xll.`, decision 0027;
    /// T-0462).
    #[serde(default)]
    pub skipped_add_in: usize,
    pub millis: u64,
    /// The spec case's `tags` (none for xlsx cases or unreadable ones); the probe milestone
    /// report reads a case's milestone tag from them (decision 0051).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Every scored cell (reports may drop passing cells when serializing). Each is written with
    /// its closeness bucket and family flag ([`crate::closeness::serialize_cells`]).
    #[serde(serialize_with = "crate::closeness::serialize_cells")]
    pub cells: Vec<CellResult>,
    /// Displayed-text verdicts, beside the values' (T-1610, decision 0068).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text: Vec<TextResult>,
    /// An xlsx case's workbook-scope defined names, upper-case name → formula text without `=`;
    /// the milestone function scope follows the names a formula calls (T-1970).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub defined_names: BTreeMap<String, String>,
    /// The calls the case's Excel-evidence cells make, arguments classified (T-0077, T-0083).
    /// Only the run's own report reads them, so they are never serialized.
    #[serde(skip)]
    pub arguments: Vec<CallSeen>,
    /// A poi case's formula cells in M8's subset and its unparsed formulas (decision 0146 item
    /// 2(b), T-0094); empty for other suites. Only the run's own report reads them.
    #[serde(skip)]
    pub reference_forms: ReferenceFormCells,
    /// The case's pivots whose layout is not Excel's (T-4124; empty without sidecar pivots).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pivot_layout: Vec<PivotLayout>,
    /// An xlsx case's saved pivots, each fresh or stale (T-4131; empty unless the suite trusts
    /// the workbook's cached values).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pivot_freshness: Vec<PivotFreshness>,
    /// The spec case's `pivot_features` catalog ids (T-4126).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pivot_features: Vec<String>,
    /// Its Excel-evidence sidecar pivot cells and how many pass (T-4126).
    #[serde(default)]
    pub pivot_cells: crate::pivot_features::PivotCellTally,
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
    /// (`graph_hides_subnormal`). A cached override sets it from its evidence probe's companion
    /// (decision 0090).
    from_graph: bool,
}

enum Prepared {
    Xlsx { bytes: Vec<u8> },
    Spec { spec: Box<CaseSpec> },
}

impl CaseResult {
    /// `case!*` when the case did not run to scoring ([`CaseStatus::did_not_run`]): how every exit
    /// measure lists such a case and `scripts/exit_owners.py` reads it as a whole case (decision
    /// 0174).
    #[must_use]
    pub fn blocked_key(&self) -> Option<String> {
        self.status.did_not_run().then(|| format!("{}!*", self.id))
    }

    /// A case `id` of `suite` with `status`, nothing checked or skipped and no cells: the start
    /// of every result, filled in with struct-update syntax.
    pub(crate) fn empty(id: &str, suite: &str, status: CaseStatus) -> CaseResult {
        CaseResult {
            id: id.into(),
            suite: suite.into(),
            status,
            message: None,
            expected_source: String::new(),
            checked: 0,
            passed: 0,
            deviations: 0,
            skipped_volatile: 0,
            skipped_add_in: 0,
            millis: 0,
            tags: Vec::new(),
            cells: Vec::new(),
            text: Vec::new(),
            defined_names: BTreeMap::new(),
            arguments: Vec::new(),
            reference_forms: ReferenceFormCells::default(),
            pivot_layout: Vec::new(),
            pivot_freshness: Vec::new(),
            pivot_features: Vec::new(),
            pivot_cells: crate::pivot_features::PivotCellTally::default(),
        }
    }

    /// A passing case that checked one passing cell and holds none, for tests.
    #[cfg(test)]
    pub(crate) fn for_test(id: &str, suite: &str) -> CaseResult {
        CaseResult {
            checked: 1,
            passed: 1,
            ..CaseResult::empty(id, suite, CaseStatus::Pass)
        }
    }
}

/// A spec case's `check` cell (or, with no `check`, a cell its expectations name): Gate A's
/// points (SPEC-0029).
#[derive(Debug, Clone, PartialEq)]
pub struct CheckCell {
    pub sheet: String,
    pub addr: Addr,
    /// The cell's formula without its leading `=`, when it holds one.
    pub formula: Option<String>,
}

/// [`run_case_values`]'s answer: the case scored as [`run_case`] scores it, its check cells, and
/// the engine's value of each, pending cells included.
#[derive(Debug, Clone)]
pub struct CaseValues {
    pub result: CaseResult,
    /// Every check cell of the case; empty for an xlsx case or one that failed to load.
    pub check: Vec<CheckCell>,
    /// The engine's value of each of `check`, in order; empty when the case did not run to
    /// scoring ([`CaseStatus::did_not_run`]).
    pub values: Vec<Value>,
}

struct PreparedCase {
    input: Prepared,
    /// A spec case's check cells (see [`CheckCell`]).
    check: Vec<CheckCell>,
    /// The workbook's file name in the oracle (sidecar `source.workbook_name`; T-2042).
    workbook_name: Option<String>,
    expectations: Vec<Expectation>,
    extent: Option<SpillExtent>,
    expected_source: String,
    skipped_volatile: usize,
    skipped_add_in: usize,
    tags: Vec<String>,
    text: Vec<TextExpectation>,
    defined_names: BTreeMap<String, String>,
    reference_forms: ReferenceFormCells,
    /// The sidecar's pivots, whose layout is compared with the engine's (T-4124).
    pivots: Vec<OraclePivot>,
    /// An xlsx case's pivots as Excel saved them, classified fresh or stale (T-4131).
    saved_pivots: Vec<SavedPivot>,
    /// The spec case's pivot catalog ids and its sidecar pivot-area cell keys (T-4126).
    pivot_features: Vec<String>,
    pivot_keys: BTreeSet<String>,
    /// An engine-export case (T-4177): its pivots' `rowItems`/`colItems` are compared with the
    /// ones the engine saves, Excel's refusal is its verdict, and a note says when the export
    /// changed since the sidecar's fill.
    export: Option<ExportOutcome>,
}

/// What an engine-export case adds to its scoring (T-4177).
#[derive(Default)]
struct ExportOutcome {
    /// Excel's error text, when the sidecar records that it would not open the file.
    refused: Option<String>,
    /// Why the result carries a note: the sidecar is of another package than the current one.
    note: Option<String>,
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
        message: Some(message),
        ..CaseResult::empty(&case.id, &suite.manifest.name, CaseStatus::InvalidCase)
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
    run_isolated(engine, suite, case, timeout, false).result
}

/// [`run_case`], also returning the engine's value of every check cell, whether or not an
/// expectation exists for it: Gate A checks pending points against the math (SPEC-0029).
#[must_use]
pub fn run_case_values(
    engine: &Arc<dyn Engine + Send>,
    suite: &Suite,
    case: &CaseFile,
    timeout: Duration,
) -> CaseValues {
    run_isolated(engine, suite, case, timeout, true)
}

/// The shared body of [`run_case`] and [`run_case_values`]; `values` reads the check cells.
fn run_isolated(
    engine: &Arc<dyn Engine + Send>,
    suite: &Suite,
    case: &CaseFile,
    timeout: Duration,
    values: bool,
) -> CaseValues {
    // An unreadable math sidecar is a broken committed file, so the case reports it as invalid.
    let prepared = MathTable::load(&math_sidecar(&case.sidecar))
        .map_err(|e| e.to_string())
        .and_then(|m| prepare(suite, case, values).map(|p| (p, m)));
    let ((mut prepared, refusal), math_table) = match prepared {
        Ok(p) => p,
        Err(e) => {
            return CaseValues {
                result: invalid(suite, case, e),
                check: Vec::new(),
                values: Vec::new(),
            }
        }
    };
    let check = if values {
        prepared.check.clone()
    } else {
        Vec::new()
    };
    // A whole-case pin (decision 0168) needs the cells after the expectations move to the worker.
    let refused_cells = refusal.as_ref().map(|_| refused(&prepared.expectations));
    let listed = refusal.is_some();
    let mut result = CaseResult {
        expected_source: prepared.expected_source.clone(),
        skipped_volatile: prepared.skipped_volatile,
        skipped_add_in: prepared.skipped_add_in,
        tags: prepared.tags.clone(),
        pivot_features: prepared.pivot_features.clone(),
        defined_names: prepared.defined_names.clone(),
        reference_forms: prepared.reference_forms.clone(),
        ..CaseResult::empty(&case.id, &suite.manifest.name, CaseStatus::Pending)
    };
    let pivot_keys = std::mem::take(&mut prepared.pivot_keys);
    let export = prepared.export.as_mut().map(std::mem::take);
    // Cases without expectations still run: a panic, timeout or load error is a crash we can
    // catch before any oracle value exists (T-0007).
    let engine = Arc::clone(engine);
    case_log::record("start", &case.id);
    let (outcome, millis) = run_in_worker(format!("case:{}", case.id), timeout, move || {
        evaluate_prepared(engine.as_ref(), &prepared, listed, values)
    });
    // A timed-out worker keeps running, so it is logged as such rather than as ended.
    let timed_out = matches!(outcome, Err((CaseStatus::Timeout, _)));
    case_log::record(if timed_out { "timeout" } else { "end" }, &case.id);
    result.millis = millis;
    let mut check_values = Vec::new();
    match outcome {
        Err((status, message)) => {
            result.status = status;
            result.message = Some(message);
        }
        Ok(Err(e)) => match (refusal, refused_cells) {
            (Some(message), Some(cells)) if e.contains(message.as_str()) => {
                result.checked = cells.len();
                result.deviations = cells.len();
                result.status = if cells.is_empty() {
                    CaseStatus::Pending
                } else {
                    CaseStatus::Pass
                };
                result.cells = cells;
                result.message = Some(e);
            }
            _ => {
                result.status = CaseStatus::EngineError;
                result.message = Some(e);
            }
        },
        Ok(Ok((cells, scored, text, arguments, got, (layout, freshness)))) => {
            check_values = got;
            result.pivot_layout = layout;
            result.pivot_freshness = freshness;
            result.text = text;
            result.arguments = arguments;
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
            result.pivot_cells = crate::pivot_features::PivotCellTally::of(&result, &pivot_keys);
            if let Some(table) = &math_table {
                attach_math(&mut result.cells, table);
            }
        }
    }
    if let Some(export) = export {
        apply_export(&mut result, export);
    }
    CaseValues {
        result,
        check,
        values: check_values,
    }
}

/// An engine-export case's refusal and note on its result: a case Excel refused fails as
/// `excel-refused` unless the engine itself did not run it.
fn apply_export(result: &mut CaseResult, export: ExportOutcome) {
    if let Some(why) = export.refused {
        if !result.status.did_not_run() {
            result.status = CaseStatus::ExcelRefused;
            result.message = Some(format!("excel-refused: {why}"));
        }
    }
    if let Some(note) = export.note {
        result.message = Some(match result.message.take() {
            Some(m) => format!("{m}; {note}"),
            None => note,
        });
    }
}

/// Runs `work` on a worker thread named `name`, with the large stack the evaluator's recursion
/// needs, isolated against panics and stopped waiting after `timeout`: one definition for every
/// harness that runs untrusted workbooks (the cell runner and the round-trip suites). Returns
/// the result, or the status and message of a worker that panicked, timed out or could not
/// start, and the elapsed milliseconds. A timed-out worker keeps running detached.
pub(crate) fn run_in_worker<T: Send + 'static>(
    name: String,
    timeout: Duration,
    work: impl FnOnce() -> T + Send + 'static,
) -> (Result<T, (CaseStatus, String)>, u64) {
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    let spawned = std::thread::Builder::new()
        .name(name)
        .stack_size(256 * 1024 * 1024) // deep dependency chains recurse in the evaluator
        .spawn(move || {
            let _ = tx.send(catch_unwind(AssertUnwindSafe(work)));
        });
    let outcome = match spawned {
        Err(e) => Err((
            CaseStatus::EngineError,
            format!("could not spawn worker: {e}"),
        )),
        Ok(_) => match rx.recv_timeout(timeout) {
            Err(_) => Err((
                CaseStatus::Timeout,
                format!("exceeded {}s", timeout.as_secs()),
            )),
            Ok(Err(panic)) => Err((CaseStatus::Panic, panic_message(panic.as_ref()))),
            Ok(Ok(value)) => Ok(value),
        },
    };
    let millis = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    (outcome, millis)
}

/// The value of a cell on a sheet the evaluated workbook does not have.
const MISSING_SHEET: &str = "<missing sheet>";

/// What a case's worker hands back: its scored cells, how many are expectations, the text
/// verdicts, the calls seen, the check cells' values, and its pivots' layout verdicts and
/// freshness.
type Evaluation = (
    Vec<CellResult>,
    usize,
    Vec<TextResult>,
    Vec<CallSeen>,
    Vec<Value>,
    (Vec<PivotLayout>, Vec<PivotFreshness>),
);

/// Evaluates a prepared case and scores it (on the worker thread, inside the panic isolation);
/// `values` also reads every check cell.
fn evaluate_prepared(
    engine: &(dyn Engine + Send),
    prepared: &PreparedCase,
    listed: bool,
    values: bool,
) -> Result<Evaluation, String> {
    let evaluated = match &prepared.input {
        Prepared::Xlsx { bytes } => engine.evaluate_xlsx(bytes, prepared.workbook_name.as_deref()),
        Prepared::Spec { spec } => engine.evaluate_spec(spec, prepared.workbook_name.as_deref()),
    }?;
    let mut cells = score(evaluated.as_ref(), &prepared.expectations, listed);
    let scored = cells.len();
    let extent = prepared.extent.iter();
    cells.extend(extent.flat_map(|e| e.unexpected_cells(evaluated.as_ref())));
    let text = text_expect::score(evaluated.as_ref(), &prepared.text);
    let sheets = evaluated.sheet_names();
    let arguments = argument_coverage::observe(evaluated.as_ref(), &sheets, &cells);
    let check_values: Vec<Value> = if values {
        prepared
            .check
            .iter()
            .map(|c| {
                evaluated
                    .value(&c.sheet, c.addr)
                    .unwrap_or_else(|| Value::Error(MISSING_SHEET.into()))
            })
            .collect()
    } else {
        Vec::new()
    };
    // An export case's engine lines are the ones it would save (T-4177).
    let lines = match prepared.export {
        Some(_) => Some(pivot_scoring::saved_lines(evaluated.as_ref())?),
        None => None,
    };
    let layout =
        pivot_scoring::compare_layouts(&prepared.pivots, evaluated.as_ref(), lines.as_deref());
    let freshness = pivot_freshness::classify(&prepared.saved_pivots, evaluated.as_ref());
    Ok((
        cells,
        scored,
        text,
        arguments,
        check_values,
        (layout, freshness),
    ))
}

fn panic_message(p: &(dyn std::any::Any + Send)) -> String {
    p.downcast_ref::<String>()
        .cloned()
        .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_else(|| "panic with non-string payload".into())
}

/// The cells of a case its whole-case `deviations` line pins (decision 0168), when the engine
/// refused it as pinned: every expectation is a deviation, with no value of its own.
fn refused(expectations: &[Expectation]) -> Vec<CellResult> {
    expectations
        .iter()
        .map(|e| CellResult {
            sheet: e.sheet.clone(),
            cell: e.addr.to_string(),
            formula: e.formula.clone(),
            anchor_formula: e.anchor_formula.clone(),
            expected: e.expected.clone(),
            actual: Value::Error("<refused>".into()),
            verdict: Verdict::Deviation,
            from_excel: e.from_excel,
            listed_deviation: true,
            graph_subnormal: false,
            kind: CellKind::Scored,
            math: None,
            target: Target::Excel,
        })
        .collect()
}

mod prepare;
mod scoring;
use prepare::prepare;
use scoring::{attach_math, score};

#[cfg(test)]
mod tests;
