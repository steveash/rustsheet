//! `rustsheet-eval` — the Excel-compatibility harness CLI.
//!
//! ```text
//! rustsheet-eval suites                         # list suites
//! rustsheet-eval run [--suite S] [--filter X]   # run + write eval/runs/latest/{report.json,report.md}
//! rustsheet-eval check                          # gate: fail on any baseline or visible-parity regression
//! rustsheet-eval check --evidence-floor         # the gate and the evidence floor from one run
//! rustsheet-eval update-baseline                # ratchet baselines and visible parity (refuses regressions)
//! rustsheet-eval evidence-floor [--update]      # ratchet functions below the evidence floor
//! rustsheet-eval show <case-id> [--all]         # per-cell detail for one case
//! rustsheet-eval snapshot <file.xlsx>           # dump stored formulas/values as JSON
//! rustsheet-eval cases <suite>                  # cases + sidecar paths as JSON (for tools)
//! ```
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};

use rustsheet_eval::baseline::{self, Baseline};
use rustsheet_eval::calibrate::compare_snapshots;
use rustsheet_eval::catalog::Catalog;
use rustsheet_eval::catalog_coverage;
use rustsheet_eval::deviations;
use rustsheet_eval::engine::{Engine, RustSheetEngine};
use rustsheet_eval::evidence_floor::{self, Lock, EVIDENCE_FLOOR, LOCK_PATH};
use rustsheet_eval::exit_exceptions::ExitExceptions;
use rustsheet_eval::fetch::fetch_suite;
use rustsheet_eval::lift_kinds;
use rustsheet_eval::milestone_exit;
use rustsheet_eval::milestone_lists::{self, MilestoneList};
use rustsheet_eval::probe_milestones::{self, Mapping, Rules};
use rustsheet_eval::report::RunReport;
use rustsheet_eval::run_meta::RunMeta;
use rustsheet_eval::runner::{run_suites, CaseResult, RunOptions};
use rustsheet_eval::suite::{BaselineMode, FetchState, Suite, SuiteKind};
use rustsheet_eval::text_expect;
use rustsheet_eval::v15_checkpoint;
use rustsheet_eval::visible_parity::{self, VisibleCount};
use rustsheet_eval::xlsx_snapshot::{read_snapshot, WorkbookSnapshot};

#[derive(Parser)]
#[command(
    name = "rustsheet-eval",
    about = "RustSheet Excel-compatibility harness"
)]
struct Cli {
    /// Repository root (defaults to the nearest ancestor containing `eval/suites`).
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::Args, Clone)]
struct Selection {
    /// Restrict to these suites (repeatable). Default: all suites (or gate suites for check).
    #[arg(long = "suite")]
    suites: Vec<String>,
    /// Only cases whose id contains this substring.
    #[arg(long)]
    filter: Option<String>,
    /// Per-case timeout in seconds.
    #[arg(long, default_value_t = 60)]
    timeout: u64,
}

#[derive(Subcommand)]
enum Cmd {
    /// List suites and case counts.
    Suites,
    /// Download pinned external suites (`[fetch]` in suite.toml) into eval/.cache/suites.
    Fetch {
        /// Suites to fetch (repeatable). Default: every external suite.
        #[arg(long = "suite")]
        suites: Vec<String>,
    },
    /// Run suites and write the report to eval/runs/latest.
    Run {
        #[command(flatten)]
        sel: Selection,
        /// Output directory (default: eval/runs/latest).
        #[arg(long)]
        out: Option<PathBuf>,
        /// Failing examples shown per function in report.md.
        #[arg(long, default_value_t = 3)]
        examples: usize,
    },
    /// Fail if anything in the committed baselines no longer passes.
    Check {
        #[command(flatten)]
        sel: Selection,
        /// Also judge the evidence floor (as `evidence-floor` does) from the same run: the other
        /// vendored suites run beside the gate suites, once (T-2761). `--timeout` applies to the
        /// whole run; standalone `evidence-floor` uses the 60 s default, as `make eval-gates` does.
        #[arg(long)]
        evidence_floor: bool,
    },
    /// Rewrite baselines to the current passing set (ratchet).
    UpdateBaseline {
        #[command(flatten)]
        sel: Selection,
        /// Permit removing previously passing items (requires PR justification).
        #[arg(long)]
        allow_regressions: bool,
        /// Also record a drop in the visible-parity lock. The steward's daily run only: a PR
        /// leaves drops to it so fix PRs do not conflict on the lock (decision 0093).
        #[arg(long)]
        lower_visible_lock: bool,
    },
    /// Fail if more in-scope functions are below the evidence floor than
    /// quality/evidence-floor.json allows, or if the lock is stale (T-0056, decision 0062).
    /// Counts the vendored suites only, so a fetched external suite cannot move it.
    EvidenceFloor {
        /// Record a lower count in the lock.
        #[arg(long)]
        update: bool,
        /// With --update, also record a higher count (needs a decision record).
        #[arg(long)]
        allow_increase: bool,
    },
    /// Print the engine's SPEC-0011 argument-kind table, or write it to
    /// eval/catalog/lift-kinds.json (T-1776).
    LiftKinds {
        /// Write the file instead of printing.
        #[arg(long)]
        write: bool,
    },
    /// Show every scored cell of one case.
    Show {
        case_id: String,
        /// Include passing cells.
        #[arg(long)]
        all: bool,
    },
    /// Print the stored formulas/values of an xlsx as JSON.
    Snapshot { path: PathBuf },
    /// Print one suite's cases as JSON, as the harness discovers them: case and sidecar paths
    /// and, for xlsx cases, what an oracle fill needs to know (tools call this instead of
    /// re-implementing discovery).
    Cases { suite: String },
    /// Oracle calibration: compare an Excel-authored xlsx's cached values with the same
    /// workbook recalculated by an oracle backend; prints per-cell verdicts as JSON.
    Compare {
        cached: PathBuf,
        recalculated: PathBuf,
    },
}

/// One line of `rustsheet-eval cases`.
#[derive(serde::Serialize)]
struct CaseListing {
    id: String,
    path: PathBuf,
    sidecar: PathBuf,
    /// xlsx only: number of formula cells (0 = nothing to score).
    formula_cells: Option<usize>,
    /// xlsx only: the suite scores this case against its own cached values when it has no sidecar.
    trusts_cached_values: Option<bool>,
    /// xlsx only: the suite's `use_cached_values`. When false its caches are no Excel evidence, so
    /// the oracle fill never lets one replace a web number (decision 0073).
    suite_uses_cached_values: Option<bool>,
    /// xlsx only: why the workbook could not be read.
    unreadable: Option<String>,
}

fn cmd_cases(root: &Path, name: &str) -> Result<ExitCode> {
    let suites = Suite::load_all(&root.join("eval/suites"))?;
    let Some(suite) = suites.iter().find(|s| s.manifest.name == name) else {
        bail!("suite {name} not found");
    };
    if suite.fetch_state() == FetchState::NotFetched {
        bail!("suite {name} is not fetched (run `rustsheet-eval fetch --suite {name}`)");
    }
    let xlsx = suite.manifest.kind == SuiteKind::Xlsx;
    let mut out = Vec::new();
    for case in suite.cases()? {
        let mut listing = CaseListing {
            id: case.id,
            path: case.path,
            sidecar: case.sidecar,
            formula_cells: None,
            trusts_cached_values: None,
            suite_uses_cached_values: xlsx.then_some(suite.manifest.use_cached_values),
            unreadable: None,
        };
        if xlsx {
            match load_snapshot(&listing.path) {
                Ok(snap) => {
                    listing.formula_cells = Some(
                        snap.sheets
                            .iter()
                            .flat_map(|s| s.cells.values())
                            .filter(|c| c.formula.is_some())
                            .count(),
                    );
                    listing.trusts_cached_values = Some(suite.manifest.trusts_cached_values(&snap));
                }
                Err(e) => listing.unreadable = Some(e.to_string()),
            }
        }
        out.push(listing);
    }
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(ExitCode::SUCCESS)
}

fn find_root(explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(r) = explicit {
        return Ok(r);
    }
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join("eval/suites").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            bail!("could not find eval/suites in any ancestor; pass --root");
        }
    }
}

fn select(root: &Path, sel: &Selection, gate_only: bool) -> Result<Vec<Suite>> {
    let all = Suite::load_all(&root.join("eval/suites"))?;
    let chosen: Vec<Suite> = all
        .into_iter()
        .filter(|s| {
            if s.fetch_state() == FetchState::NotFetched {
                eprintln!(
                    "skipping {}: not fetched (run `rustsheet-eval fetch --suite {}`)",
                    s.manifest.name, s.manifest.name
                );
                return false;
            }
            if sel.suites.is_empty() {
                !gate_only || s.manifest.gate
            } else {
                sel.suites.contains(&s.manifest.name)
            }
        })
        .collect();
    if chosen.is_empty() {
        bail!("no suites selected");
    }
    Ok(chosen)
}

fn execute(suites: &[Suite], sel: &Selection) -> Vec<CaseResult> {
    // Panics are captured per case; keep the default hook from spamming stderr.
    std::panic::set_hook(Box::new(|_| {}));
    let engine: Arc<dyn Engine + Send> = Arc::new(RustSheetEngine);
    let opts = RunOptions {
        timeout: Duration::from_secs(sel.timeout),
        filter: sel.filter.clone(),
    };
    run_suites(&engine, suites, &opts)
}

/// Fails unless every suite's `deviations` lines name an existing case and cite an accepted
/// `Status: DEVIATION` semantics entry (decision 0036).
fn check_deviations(root: &Path, suites: &[Suite]) -> Result<()> {
    for suite in suites {
        deviations::load_checked(suite, root)
            .with_context(|| format!("suite {}", suite.manifest.name))?;
    }
    Ok(())
}

/// The frozen milestone probe lists (decision 0044), the probe-milestone rules (decision 0051)
/// and the named exit exceptions (decision 0087).
struct Milestones {
    lists: Vec<MilestoneList>,
    rules: Rules,
    exceptions: ExitExceptions,
}

/// Loads the frozen lists and the probe-milestone mapping, and fails when a line of either names
/// a case that no suite discovers, or a cell line names a cell its case holds no expectation for.
/// A malformed line of the exit exceptions fails too.
fn load_milestones(root: &Path, catalog: &Catalog) -> Result<Milestones> {
    let lists = milestone_lists::load_all(root)?;
    let exceptions = ExitExceptions::load(root)?;
    let mapping = Mapping::load(root)?;
    if !lists.is_empty() || mapping != Mapping::default() {
        let mut files = BTreeMap::new();
        for suite in Suite::load_all(&root.join("eval/suites"))? {
            files.extend(suite.cases()?.into_iter().map(|c| (c.id.clone(), c)));
        }
        let known: BTreeSet<String> = files.keys().cloned().collect();
        milestone_lists::check_known(&lists, &known)?;
        mapping.check_known(&known)?;
        // A cell line must name a cell its case scores (T-1568).
        let mut expectations = BTreeMap::new();
        for (case, _) in mapping.cells.keys() {
            if let Some(file) = files.get(case) {
                let cells = rustsheet_eval::case::expectation_cells(file)
                    .map_err(|e| anyhow!("{case}: {e}"))?;
                expectations.insert(case.clone(), cells);
            }
        }
        mapping.check_cells(&expectations)?;
    }
    let rules = Rules::new(&lists, mapping, catalog);
    Ok(Milestones {
        lists,
        rules,
        exceptions,
    })
}

/// Fills the report's milestone sections from the run's `results`.
fn score_milestones(
    milestones: &Milestones,
    catalog: &Catalog,
    suites: &[Suite],
    results: &[CaseResult],
    report: &mut RunReport,
) {
    let modes: BTreeMap<String, BaselineMode> = suites
        .iter()
        .map(|s| (s.manifest.name.clone(), s.manifest.baseline))
        .collect();
    report.milestone_lists = milestones
        .lists
        .iter()
        .map(|l| milestone_lists::summarize(l, results, &modes))
        .collect();
    let probes_mode = modes
        .get(probe_milestones::SUITE)
        .copied()
        .unwrap_or(BaselineMode::Cells);
    report.probe_milestones = probe_milestones::summarize(&milestones.rules, results, probes_mode);
    if let Some(scope) = report.function_scope.as_mut() {
        scope.apply_exceptions(&milestones.exceptions);
    }
    let generated: BTreeMap<String, Vec<String>> = suites
        .iter()
        .filter(|s| s.manifest.origin == "generated")
        .map(|s| (s.manifest.name.clone(), s.manifest.milestone.clone()))
        .collect();
    report.catalog_coverage =
        catalog_coverage::summarize(catalog, results, &report.evidence, &generated);
    let generated_names: Vec<&str> = generated.keys().map(String::as_str).collect();
    report.v15_checkpoint = v15_checkpoint::summarize(catalog, results, report, &generated_names);
    report.milestone_exit = milestone_exit::summarize(report);
}

fn print_summary(report: &RunReport) {
    println!(
        "{:<28} {:>6} {:>9} {:>9} {:>10} {:>8}",
        "suite", "cases", "cells", "passed", "deviations", "pass%"
    );
    for s in &report.suites {
        println!(
            "{:<28} {:>6} {:>9} {:>9} {:>10} {:>7.2}%",
            s.name,
            s.cases,
            s.cells,
            s.cells_passed,
            s.deviations,
            100.0 * s.cell_pass_rate()
        );
        if !s.graph_subnormal_passes.is_empty() {
            println!(
                "  note: {} cell(s) passed by decision 0042 (graph 0 vs kept subnormal; each has a FALSE (x)=0 companion)",
                s.graph_subnormal_passes.len()
            );
        }
        for line in &s.stale_deviations {
            println!("  warning: stale deviations line (the cell matches Excel): {line}");
        }
    }
    for l in report.milestone_lists.iter().filter(|l| l.cases_run > 0) {
        println!(
            "milestone list {}: {}/{} cases run, {} cells, {} bit-exact, {} deviations, {} known failures in {} cases",
            l.name,
            l.cases_run,
            l.cases,
            l.cells,
            l.bit_exact,
            l.deviations,
            l.known_failures,
            l.failing_cases.len()
        );
    }
    if let Some(r) = report.probe_milestones.first() {
        println!(
            "probe milestone {} (decision 0051): {} cases, {} cells, {} bit-exact, {} deviations, {} known failures",
            r.milestone, r.cases, r.cells, r.bit_exact, r.deviations, r.known_failures
        );
    }
}

fn baseline_path(root: &Path, suite: &str) -> PathBuf {
    root.join("eval/baseline").join(format!("{suite}.txt"))
}

/// Holds `count` to the visible-parity lock, recording it on `update` when
/// [`visible_parity::judge`] allows (T-0061, decisions 0088 and 0093); returns whether the gate
/// passes.
fn visible_parity_gate(
    root: &Path,
    count: VisibleCount,
    update: Option<visible_parity::Update>,
) -> Result<bool> {
    use visible_parity::{Judgement, Lock, LOCK_PATH};
    let lock = Lock::read(root)?;
    println!(
        "visible parity: {} in-scope cells visibly off ({} in new cases), lock {}",
        count.cells, count.new_coverage, lock.visibly_off_cells
    );
    Ok(match visible_parity::judge(lock, count, update) {
        Judgement::Holds => true,
        Judgement::Record => {
            Lock {
                visibly_off_cells: count.cells,
            }
            .write(root)?;
            println!("  updated {LOCK_PATH}: commit it");
            true
        }
        Judgement::Regressed => {
            println!("  REGRESSION: more cells are visibly off than {LOCK_PATH} allows; the report's \"Visibly off\" section names the functions");
            false
        }
    })
}

/// The baseline gate (`update` `None`) or the baseline ratchet; with `floor`, also the evidence
/// floor judged from the same run, so `check --evidence-floor` passes exactly when `check` and
/// `evidence-floor` both pass (T-2761).
fn cmd_check(
    root: &Path,
    sel: &Selection,
    update: Option<visible_parity::Update>,
    floor: bool,
) -> Result<ExitCode> {
    if sel.filter.is_some() {
        bail!(
            "--filter is not allowed for baseline operations (partial runs look like regressions)"
        );
    }
    if floor && !sel.suites.is_empty() {
        bail!("--evidence-floor counts every vendored suite; it cannot be combined with --suite");
    }
    let gate = select(root, sel, true)?;
    check_deviations(root, &gate)?;
    let catalog = Catalog::load(root)?;
    let milestones = load_milestones(root, &catalog)?;
    let gate_len = gate.len();
    let run = if floor {
        evidence_floor::plan_run(gate, Suite::load_all(&root.join("eval/suites"))?)
    } else {
        gate
    };
    // `plan_run` keeps the gate suites first, unchanged.
    let suites = &run[..gate_len];
    let mut results = execute(&run, sel);
    let floor_results = floor.then(|| evidence_floor::floor_results(&run, &results));
    if floor {
        // The baseline gate judges the gate suites' results only, as a plain `check` does.
        results.retain(|r| suites.iter().any(|s| s.manifest.name == r.suite));
    }
    let mut report = RunReport::build(
        RustSheetEngine.name(),
        &results,
        &RustSheetEngine.implemented_functions(),
        &catalog,
    );
    score_milestones(&milestones, &catalog, suites, &results, &mut report);
    print_summary(&report);
    let mut failed = false;
    let mut visible = VisibleCount::default();
    for suite in suites {
        if suite.manifest.baseline == BaselineMode::None {
            continue;
        }
        let name = &suite.manifest.name;
        let mine: Vec<&CaseResult> = results.iter().filter(|r| &r.suite == name).collect();
        let current = Baseline::from_results(suite.manifest.baseline, &mine);
        let path = baseline_path(root, name);
        let old = Baseline::read(&path)?;
        if suite.manifest.fetch.is_none() {
            for r in &mine {
                visible.add(&catalog, r, !old.checked.contains_key(&r.id));
            }
        }
        let delta = baseline::diff(&old, &current);
        println!(
            "[{name}] known failures {} -> {}, text {} -> {}  (fixed {}, new-coverage failures {}, REGRESSIONS {}, new cases {}, removed cases {})",
            old.failing.len(),
            current.failing.len(),
            old.text_failing.len(),
            current.text_failing.len(),
            delta.fixed.len(),
            delta.new_failures.len(),
            delta.regressions.len(),
            delta.new_cases.len(),
            delta.removed_cases.len()
        );
        for r in delta.regressions.iter().take(25) {
            println!("  REGRESSION {r}");
        }
        match update {
            None => {
                if delta.is_dirty() {
                    failed = true;
                    if delta.regressions.is_empty() {
                        println!("  baseline is out of date: run `make eval-baseline` and commit eval/baseline/");
                    }
                }
            }
            Some(u) => {
                if !delta.regressions.is_empty() && !u.allow_regressions {
                    failed = true;
                    println!("  refusing to record regressions without --allow-regressions");
                } else {
                    current.write(&path)?;
                }
            }
        }
    }
    // A partial run (`--suite`) would undercount, so only a full gated run is held to the lock.
    if sel.suites.is_empty() && !visible_parity_gate(root, visible, update)? {
        failed = true;
    }
    if let Some(floor_results) = floor_results {
        if !evidence_floor_gate(root, &catalog, &floor_results, false, false)? {
            failed = true;
        }
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn cmd_lift_kinds(root: &Path, write: bool) -> Result<ExitCode> {
    let text = lift_kinds::render()?;
    if write {
        let path = root.join(lift_kinds::LIFT_KINDS_PATH);
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        println!("wrote {}", lift_kinds::LIFT_KINDS_PATH);
    } else {
        print!("{text}");
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_evidence_floor(root: &Path, update: bool, allow_increase: bool) -> Result<ExitCode> {
    let sel = Selection {
        suites: Suite::load_all(&root.join("eval/suites"))?
            .into_iter()
            .filter(evidence_floor::counts)
            .map(|s| s.manifest.name)
            .collect(),
        filter: None,
        timeout: 60,
    };
    let suites = select(root, &sel, false)?;
    let catalog = Catalog::load(root)?;
    let results = execute(&suites, &sel);
    Ok(
        if evidence_floor_gate(root, &catalog, &results, update, allow_increase)? {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    )
}

/// Holds the number of functions below the floor in `results` (a run of the vendored suites) to
/// [`LOCK_PATH`], recording it on `update` when [`evidence_floor::may_record`] allows; returns
/// whether the gate passes (decision 0062).
fn evidence_floor_gate(
    root: &Path,
    catalog: &Catalog,
    results: &[CaseResult],
    update: bool,
    allow_increase: bool,
) -> Result<bool> {
    let report = RunReport::build(
        RustSheetEngine.name(),
        results,
        &RustSheetEngine.implemented_functions(),
        catalog,
    );
    let count = report.below_floor.len();
    let lock = Lock::read(root)?;
    let verdict = evidence_floor::judge(lock, count);
    println!(
        "evidence floor: {count} in-scope functions with fewer than {EVIDENCE_FLOOR} Excel cells (lock {} at floor {})",
        lock.below_floor, lock.floor
    );
    let new_lock = Lock {
        floor: EVIDENCE_FLOOR,
        below_floor: count,
    };
    let ok = match (verdict, update) {
        (evidence_floor::Verdict::Unchanged, _) => true,
        (_, true) if !evidence_floor::may_record(verdict, allow_increase) => {
            println!("  refusing to record an increase or a new floor without --allow-increase (needs a decision record)");
            false
        }
        (_, true) => {
            new_lock.write(root)?;
            println!("  updated {LOCK_PATH}: commit it");
            true
        }
        (evidence_floor::Verdict::Improved, false) => {
            println!("  fewer functions are below the floor: run `make eval-baseline` and commit {LOCK_PATH}");
            false
        }
        (evidence_floor::Verdict::Regressed, false) => {
            println!("  REGRESSION: more functions are below the floor than {LOCK_PATH} allows; the report's \"Below the evidence floor\" list names them");
            false
        }
        (evidence_floor::Verdict::FloorChanged, false) => {
            println!("  {LOCK_PATH} was taken at another floor: rerun with --update --allow-increase under a decision record");
            false
        }
    };
    Ok(ok)
}

fn cmd_show(root: &Path, case_id: &str, all: bool) -> Result<ExitCode> {
    let suite_name = case_id.split('/').next().unwrap_or_default().to_string();
    let sel = Selection {
        suites: vec![suite_name],
        filter: Some(case_id.to_string()),
        timeout: 120,
    };
    let suites = select(root, &sel, false)?;
    let results = execute(&suites, &sel);
    let Some(r) = results.iter().find(|r| r.id == case_id) else {
        bail!("case {case_id} not found");
    };
    println!(
        "{}  status={:?}  checked={} passed={} deviations={}  source={}",
        r.id, r.status, r.checked, r.passed, r.deviations, r.expected_source
    );
    if let Some(m) = &r.message {
        println!("message: {m}");
    }
    for c in r.cells.iter().filter(|c| all || !c.verdict.is_pass()) {
        println!(
            "{:<20} {:?}\n    formula : ={}\n    expected: {}\n    actual  : {}",
            c.key(),
            c.verdict,
            c.formula.as_deref().unwrap_or(""),
            c.expected,
            c.actual
        );
    }
    let mut text = String::new();
    text_expect::write_case(&r.text, all, &mut text);
    print!("{text}");
    Ok(ExitCode::SUCCESS)
}

fn load_snapshot(path: &Path) -> Result<WorkbookSnapshot> {
    let bytes = std::fs::read(path).with_context(|| path.display().to_string())?;
    Ok(read_snapshot(&bytes)?)
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    if let Cmd::Snapshot { path } = &cli.cmd {
        println!("{}", serde_json::to_string_pretty(&load_snapshot(path)?)?);
        return Ok(ExitCode::SUCCESS);
    }
    if let Cmd::Compare {
        cached,
        recalculated,
    } = &cli.cmd
    {
        let comparison = compare_snapshots(&load_snapshot(cached)?, &load_snapshot(recalculated)?);
        println!("{}", serde_json::to_string(&comparison)?);
        return Ok(ExitCode::SUCCESS);
    }
    let root = find_root(cli.root)?;
    match cli.cmd {
        Cmd::Suites => {
            for s in Suite::load_all(&root.join("eval/suites"))? {
                let n = s.cases().map_or(0, |c| c.len());
                let cases = match s.fetch_state() {
                    FetchState::NotFetched => rustsheet_eval::run_meta::NOT_FETCHED.to_string(),
                    _ => format!("{n} cases"),
                };
                println!(
                    "{:<28} {:>12}  gate={:<5} license={:<20} {}",
                    s.manifest.name,
                    cases,
                    s.manifest.gate,
                    s.manifest.license,
                    s.manifest.description
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Fetch { suites } => {
            let all = Suite::load_all(&root.join("eval/suites"))?;
            let mut fetched = 0;
            for s in all.iter().filter(|s| {
                s.manifest.fetch.is_some()
                    && (suites.is_empty() || suites.contains(&s.manifest.name))
            }) {
                let files =
                    fetch_suite(s).with_context(|| format!("fetching {}", s.manifest.name))?;
                println!(
                    "fetched {} ({files} files) -> {}",
                    s.manifest.name,
                    s.cache_dir.display()
                );
                fetched += 1;
            }
            if fetched == 0 {
                bail!("no matching external suites");
            }
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Run { sel, out, examples } => {
            let suites = select(&root, &sel, false)?;
            check_deviations(&root, &suites)?;
            let catalog = Catalog::load(&root)?;
            let milestones = load_milestones(&root, &catalog)?;
            let started = std::time::SystemTime::now();
            let results = execute(&suites, &sel);
            let mut report = RunReport::build(
                RustSheetEngine.name(),
                &results,
                &RustSheetEngine.implemented_functions(),
                &catalog,
            );
            score_milestones(&milestones, &catalog, &suites, &results, &mut report);
            let all = Suite::load_all(&root.join("eval/suites"))?;
            report.run = RunMeta::new(&root, started, &all, &suites);
            let out = out.unwrap_or_else(|| root.join("eval/runs/latest"));
            std::fs::create_dir_all(&out)?;
            std::fs::write(
                out.join("report.json"),
                serde_json::to_string_pretty(&report)?,
            )?;
            std::fs::write(out.join("report.md"), report.to_markdown(examples))?;
            print_summary(&report);
            println!("report: {}", out.join("report.md").display());
            Ok(ExitCode::SUCCESS)
        }
        Cmd::Check {
            sel,
            evidence_floor,
        } => cmd_check(&root, &sel, None, evidence_floor),
        Cmd::UpdateBaseline {
            sel,
            allow_regressions,
            lower_visible_lock,
        } => cmd_check(
            &root,
            &sel,
            Some(visible_parity::Update {
                allow_regressions,
                lower: lower_visible_lock,
            }),
            false,
        ),
        Cmd::EvidenceFloor {
            update,
            allow_increase,
        } => cmd_evidence_floor(&root, update, allow_increase),
        Cmd::LiftKinds { write } => cmd_lift_kinds(&root, write),
        Cmd::Show { case_id, all } => cmd_show(&root, &case_id, all),
        Cmd::Cases { suite } => cmd_cases(&root, &suite),
        Cmd::Snapshot { .. } | Cmd::Compare { .. } => unreachable!("handled above"),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
