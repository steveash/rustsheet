//! Case preparation (T-4598): a suite's case file read into a [`PreparedCase`], its expectations
//! built from the oracle sidecar, the cached values or the spec, and its cached overrides and
//! deviations applied. Moved out of runner.rs unchanged.

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Duration;

use super::{run_in_worker, CheckCell, Expectation, ExportOutcome, Prepared, PreparedCase};
use crate::case::{
    canonical_key, cell_key, is_excel_evidence, kept_nonzero_formulas, load_oracle,
    split_qualified, CaseSpec, CASE_FORMAT,
};
use crate::cellref::Addr;
use crate::deviations::{self, Pin};
use crate::export_case;
use crate::formula_calls::calls_add_in;
use crate::overrides;
use crate::pivot_freshness;
use crate::pivot_scoring;
use crate::poi_m8_subset::{self, ReferenceFormCells};
use crate::spill_extent::SpillExtent;
use crate::suite::{CaseFile, Suite, SuiteKind};
use crate::text_expect;
use crate::value::Value;
use crate::volatile::{pinned_clock_serial, scorable_cells, spec_volatile_cells, FormulaCell};
use crate::xlsx_snapshot::{is_undecoded, read_snapshot};

/// Resolves the oracle value for (sheet, cell).
type Lookup = Box<dyn Fn(&str, Addr) -> Option<Value>>;

/// The case ready to run, and the refusal message its whole-case `deviations` line pins, if any.
/// `values` also lists the spec case's check cells ([`run_case_values`]); a plain run skips it.
pub(super) fn prepare(
    suite: &Suite,
    case: &CaseFile,
    values: bool,
) -> Result<(PreparedCase, Option<String>), String> {
    let mut prepared = match suite.manifest.kind {
        SuiteKind::Xlsx => prepare_xlsx(suite, case),
        SuiteKind::Spec => prepare_spec(case, values),
        SuiteKind::Export => prepare_export(suite, case),
        // `Suite::load_all` keeps round-trip suites out of every cell run (SPEC-0033 D8).
        SuiteKind::Roundtrip => {
            Err("a roundtrip suite is run by `rustsheet-eval roundtrip`".into())
        }
    }?;
    let refusal = pin_deviations(suite, case, &mut prepared.expectations)?;
    Ok((prepared, refusal))
}

/// The case's id relative to its suite root, as `cached_overrides` and `deviations` lines name it.
fn relative_case_id<'a>(suite: &Suite, case: &'a CaseFile) -> &'a str {
    let prefix = format!("{}/", suite.manifest.name);
    case.id.strip_prefix(&prefix).unwrap_or(&case.id)
}

/// Pins the by-design value of every cell the suite's `deviations` file lists for this case
/// (decision 0036), and returns the refusal message a whole-case line pins (decision 0168). A
/// line naming a cell that is not scored is an error, as for overrides.
fn pin_deviations(
    suite: &Suite,
    case: &CaseFile,
    expectations: &mut [Expectation],
) -> Result<Option<String>, String> {
    let rel = relative_case_id(suite, case);
    let mut refusal = None;
    for d in deviations::load(suite)
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|d| d.case == rel)
    {
        match d.pin {
            Pin::Cell { sheet, addr, value } => {
                let target = expectations
                    .iter_mut()
                    .find(|e| e.sheet == sheet && e.addr == addr)
                    .ok_or_else(|| format!("deviation for unscored cell {sheet}!{addr}"))?;
                target.pinned = Some(value);
            }
            Pin::Refusal(message) => refusal = Some(message),
        }
    }
    Ok(refusal)
}

fn prepare_xlsx(suite: &Suite, case: &CaseFile) -> Result<PreparedCase, String> {
    let bytes = std::fs::read(&case.path).map_err(|e| e.to_string())?;
    prepare_xlsx_bytes(suite, case, bytes)
}

/// [`prepare_xlsx`] of the workbook `bytes`, the case file's own or the package an export case
/// saves.
fn prepare_xlsx_bytes(
    suite: &Suite,
    case: &CaseFile,
    bytes: Vec<u8>,
) -> Result<PreparedCase, String> {
    let snapshot = read_snapshot(&bytes).map_err(|e| e.to_string())?;
    // Every poi workbook, pending ones too: a crashed one holding a subset formula is listed. The
    // parser reads third-party text here, outside the evaluation's panic isolation, so a panic
    // leaves the workbook unclassified instead of ending the run.
    let reference_forms = if suite.manifest.name == poi_m8_subset::SUITE {
        catch_unwind(AssertUnwindSafe(|| ReferenceFormCells::classify(&snapshot)))
            .unwrap_or_default()
    } else {
        ReferenceFormCells::default()
    };
    // Only cells the suite trusts as Excel's say whether a pivot's cache reproduces them (T-4131).
    let saved_pivots = if suite.manifest.trusts_cached_values(&snapshot) {
        pivot_freshness::saved_pivots(&snapshot)
    } else {
        Vec::new()
    };
    let oracle = load_oracle(case)?;
    let workbook_name = oracle.as_ref().and_then(|o| o.source.workbook_name.clone());
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
            check: Vec::new(),
            workbook_name,
            expectations: Vec::new(),
            extent: None,
            expected_source: "none".into(),
            skipped_volatile: 0,
            skipped_add_in: 0,
            tags: Vec::new(),
            text: Vec::new(),
            defined_names: BTreeMap::new(),
            reference_forms,
            pivots: Vec::new(),
            saved_pivots: Vec::new(),
            pivot_features: Vec::new(),
            pivot_keys: BTreeSet::new(),
            export: None,
        });
    };
    let (scorable, skipped_volatile) =
        scorable_cells(&snapshot, pinned_clock_serial(&snapshot.sheets).is_some());
    let (add_in, scorable): (Vec<_>, Vec<_>) =
        scorable.into_iter().partition(|(sheet, _, cell)| {
            cell.formula.as_deref().is_some_and(calls_add_in)
                || cell
                    .anchor_formula(&snapshot.sheets, sheet)
                    .is_some_and(|f| calls_add_in(&f))
        });
    let skipped_add_in = add_in.len();
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
        check: Vec::new(),
        workbook_name,
        expectations,
        extent: None,
        expected_source: source,
        skipped_volatile,
        skipped_add_in,
        tags: Vec::new(),
        text: Vec::new(),
        defined_names,
        reference_forms,
        pivots: Vec::new(),
        saved_pivots,
        pivot_features: Vec::new(),
        pivot_keys: BTreeSet::new(),
        export: None,
    })
}

/// Replaces the expectations a suite's `cached_overrides` file names for this case with their
/// golden values (decision 0032). An override naming a cell that is not scored is an error, so a
/// stale line cannot linger unnoticed.
pub(super) fn apply_cached_overrides(
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
        let evidence = overrides::evidence(o, suites_dir)?;
        let target = expectations
            .iter_mut()
            .find(|e| e.sheet == o.sheet && e.addr == o.addr)
            .ok_or_else(|| format!("cached override for unscored cell {}!{}", o.sheet, o.addr))?;
        target.expected = evidence.value;
        target.from_graph = evidence.kept_nonzero;
    }
    Ok(expectations)
}

/// Every formula cell of a spec case, as (sheet, cell, formula text).
fn spec_formulas(spec: &CaseSpec) -> Vec<FormulaCell> {
    spec.sheets
        .iter()
        .flat_map(|sheet| {
            sheet.cells.iter().filter_map(|(a1, input)| {
                Some((
                    sheet.name.clone(),
                    Addr::parse(a1)?,
                    input.formula()?.to_string(),
                ))
            })
        })
        .collect()
}

pub(super) fn prepare_spec(case: &CaseFile, values: bool) -> Result<PreparedCase, String> {
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
    let mut workbook_name = None;
    // Formulas whose excel-graph companion `(x)=0` is FALSE: Excel kept `x` non-zero (decision 0042).
    let mut kept_nonzero = BTreeSet::new();
    // Sidecar pivot-area cells (T-4124), kept apart from `oracle_cells` (pivot_scoring.rs).
    let (mut pivots, mut pivot_keys) = (Vec::new(), BTreeSet::new());
    if let Some(mut o) = load_oracle(case)? {
        pivots = std::mem::take(&mut o.pivots);
        kept_nonzero = kept_nonzero_formulas(&spec, &o);
        source.clone_from(&o.source.kind);
        workbook_name = o.source.workbook_name;
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
        pivot_keys = pivot_scoring::add_expectations(&pivots, &oracle_cells, &mut expected);
    }
    let formula_of = |sheet: &str, a1: &str| {
        spec.sheets
            .iter()
            .find(|s| s.name == sheet)
            .and_then(|s| s.cells.get(a1))
            .and_then(|input| input.formula().map(str::to_string))
    };
    let formulas = spec_formulas(&spec);
    // Spec workbooks have no METADATA sheet, so the clock is never pinned: NOW/TODAY stay skipped
    // unless the case lists the cell as run-invariant.
    let volatile = spec_volatile_cells(&spec.invariant, &formulas)?;
    let wanted: Vec<String> = if spec.check.is_empty() {
        expected.keys().cloned().collect()
    } else {
        let listed: BTreeSet<&String> = spec.check.iter().collect();
        let extra = pivot_keys.iter().filter(|k| !listed.contains(k));
        spec.check.iter().chain(extra).cloned().collect()
    };
    let mut expectations = Vec::new();
    let mut check = Vec::new();
    let mut skipped_volatile = 0;
    let mut skipped_add_in = 0;
    for q in wanted {
        let (sheet, a1) = split_qualified(&q).ok_or_else(|| format!("bad cell key {q}"))?;
        let addr = Addr::parse(&a1).ok_or_else(|| format!("bad cell key {q}"))?;
        let key = cell_key(&sheet, addr);
        let formula = formula_of(&sheet, &a1);
        if values {
            check.push(CheckCell {
                sheet: sheet.clone(),
                addr,
                formula: formula.clone(),
            });
        }
        let Some(value) = expected.get(&key).filter(|v| !is_undecoded(v)).cloned() else {
            continue; // pending oracle for this cell (or an undecoded rich error)
        };
        let from_graph = formula.as_deref().is_some_and(|f| kept_nonzero.contains(f));
        if volatile.contains(&(sheet.clone(), addr)) {
            skipped_volatile += 1;
            continue;
        }
        if formula.as_deref().is_some_and(calls_add_in) {
            skipped_add_in += 1;
            continue;
        }
        expectations.push(Expectation {
            sheet,
            addr,
            formula,
            anchor_formula: None,
            expected: value,
            from_excel: oracle_is_excel
                && (oracle_cells.contains(&key) || pivot_keys.contains(&key)),
            pinned: None,
            from_graph,
        });
    }
    Ok(PreparedCase {
        extent: oracle_is_excel
            .then(|| SpillExtent::new(&spec, &expected, &oracle_cells, &formulas)),
        tags: spec.tags.clone(),
        pivot_features: spec.pivot_features.clone(),
        pivot_keys,
        text: text_expect::with_pivot_text(text_expect::expectations(&spec, case)?, &pivots),
        defined_names: BTreeMap::new(),
        reference_forms: ReferenceFormCells::default(),
        pivots,
        saved_pivots: Vec::new(),
        input: Prepared::Spec {
            spec: Box::new(spec),
        },
        check,
        workbook_name,
        expectations,
        expected_source: source,
        skipped_volatile,
        skipped_add_in,
        export: None,
    })
}

/// How long building and saving one export case may take, outside the case's own timeout.
const EXPORT_TIMEOUT: Duration = Duration::from_secs(120);

/// An engine-export case (T-4177): the package its workbook saves ([`export_case::export`], on
/// an isolated worker), scored as an xlsx case against the sidecar's values, with every cell of
/// the sidecar's pivot areas an expectation too, as a spec case scores them.
fn prepare_export(suite: &Suite, case: &CaseFile) -> Result<PreparedCase, String> {
    let tags = export_case::read(&case.path)?.tags;
    let path = case.path.clone();
    let (outcome, _) = run_in_worker(format!("export:{}", case.id), EXPORT_TIMEOUT, move || {
        export_case::export(&path)
    });
    let bytes = match outcome {
        Ok(result) => result.map_err(|e| format!("export failed: {e}"))?,
        Err((status, message)) => return Err(format!("export {status:?}: {message}")),
    };
    let sha = export_case::sha256_hex(&bytes);
    let mut prepared = prepare_xlsx_bytes(suite, case, bytes)?;
    prepared.tags = tags;
    let mut export = ExportOutcome::default();
    if let Some(oracle) = load_oracle(case)? {
        let filled = oracle.source.input_sha256.as_deref().unwrap_or("none");
        if filled != sha {
            export.note = Some(format!(
                "the export changed since the oracle fill (sidecar {filled}, now {sha}): \
                 refill with `rustsheet-oracle fill --force`"
            ));
        }
        export.refused = oracle.refused;
        let pivots = oracle.pivots;
        let given: BTreeSet<String> = (prepared.expectations.iter())
            .map(|e| cell_key(&e.sheet, e.addr))
            .collect();
        let from_excel = is_excel_evidence(&oracle.source.kind);
        for (sheet, addr, expected) in pivot_scoring::pivot_cells(&pivots) {
            let key = cell_key(&sheet, addr);
            if given.contains(&key) || !prepared.pivot_keys.insert(key) {
                continue;
            }
            prepared.expectations.push(Expectation {
                sheet,
                addr,
                formula: None,
                anchor_formula: None,
                expected,
                from_excel,
                pinned: None,
                from_graph: false,
            });
        }
        prepared.text = text_expect::with_pivot_text(prepared.text, &pivots);
        prepared.pivots = pivots;
    }
    prepared.export = Some(export);
    Ok(prepared)
}
