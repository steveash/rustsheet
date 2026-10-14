//! `rustsheet-eval` — the Excel-compatibility harness CLI.
//!
//! ```text
//! rustsheet-eval suites                         # list suites
//! rustsheet-eval run [--suite S] [--filter X]   # run + write eval/runs/latest/{report.json,report.md}
//! rustsheet-eval check                          # gate: fail on any baseline regression
//! rustsheet-eval update-baseline                # ratchet baselines up (refuses regressions)
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

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use rustsheet_eval::baseline::{self, Baseline};
use rustsheet_eval::calibrate::compare_snapshots;
use rustsheet_eval::catalog::Catalog;
use rustsheet_eval::deviations;
use rustsheet_eval::engine::{Engine, RustSheetEngine};
use rustsheet_eval::fetch::fetch_suite;
use rustsheet_eval::milestone_lists::{self, MilestoneList};
use rustsheet_eval::probe_milestones::{self, Mapping, Rules};
use rustsheet_eval::report::RunReport;
use rustsheet_eval::runner::{run_suites, CaseResult, RunOptions};
use rustsheet_eval::suite::{BaselineMode, FetchState, Suite, SuiteKind};
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
    },
    /// Rewrite baselines to the current passing set (ratchet).
    UpdateBaseline {
        #[command(flatten)]
        sel: Selection,
        /// Permit removing previously passing items (requires PR justification).
        #[arg(long)]
        allow_regressions: bool,
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

/// The frozen milestone probe lists (decision 0044) and the probe-milestone rules (decision 0051).
struct Milestones {
    lists: Vec<MilestoneList>,
    rules: Rules,
}

/// Loads the frozen lists and the probe-milestone mapping, and fails when a line of either names
/// a case that no suite discovers.
fn load_milestones(root: &Path, catalog: &Catalog) -> Result<Milestones> {
    let lists = milestone_lists::load_all(root)?;
    let mapping = Mapping::load(root)?;
    if !lists.is_empty() || mapping != Mapping::default() {
        let mut known = BTreeSet::new();
        for suite in Suite::load_all(&root.join("eval/suites"))? {
            known.extend(suite.cases()?.into_iter().map(|c| c.id));
        }
        milestone_lists::check_known(&lists, &known)?;
        mapping.check_known(&known)?;
    }
    let rules = Rules::new(&lists, mapping, catalog);
    Ok(Milestones { lists, rules })
}

/// Fills the report's milestone sections from the run's `results`.
///
/// # Errors
/// When a cell line of the probe-milestone mapping names a cell its case does not check.
fn score_milestones(
    milestones: &Milestones,
    suites: &[Suite],
    results: &[CaseResult],
    report: &mut RunReport,
) -> Result<()> {
    milestones.rules.check_cells(results)?;
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
    Ok(())
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

fn cmd_check(root: &Path, sel: &Selection, update: Option<bool>) -> Result<ExitCode> {
    if sel.filter.is_some() {
        bail!(
            "--filter is not allowed for baseline operations (partial runs look like regressions)"
        );
    }
    let suites = select(root, sel, true)?;
    check_deviations(root, &suites)?;
    let catalog = Catalog::load(root)?;
    let milestones = load_milestones(root, &catalog)?;
    let results = execute(&suites, sel);
    let mut report = RunReport::build(
        RustSheetEngine.name(),
        &results,
        &RustSheetEngine.implemented_functions(),
        &catalog,
    );
    score_milestones(&milestones, &suites, &results, &mut report)?;
    print_summary(&report);
    let mut failed = false;
    for suite in &suites {
        if suite.manifest.baseline == BaselineMode::None {
            continue;
        }
        let name = &suite.manifest.name;
        let mine: Vec<&CaseResult> = results.iter().filter(|r| &r.suite == name).collect();
        let current = Baseline::from_results(suite.manifest.baseline, &mine);
        let path = baseline_path(root, name);
        let old = Baseline::read(&path)?;
        let delta = baseline::diff(&old, &current);
        println!(
            "[{name}] known failures {} -> {}  (fixed {}, new-coverage failures {}, REGRESSIONS {}, new cases {}, removed cases {})",
            old.failing.len(),
            current.failing.len(),
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
            Some(allow) => {
                if !delta.regressions.is_empty() && !allow {
                    failed = true;
                    println!("  refusing to record regressions without --allow-regressions");
                } else {
                    current.write(&path)?;
                }
            }
        }
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
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
                    FetchState::NotFetched => "not fetched".to_string(),
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
            let results = execute(&suites, &sel);
            let mut report = RunReport::build(
                RustSheetEngine.name(),
                &results,
                &RustSheetEngine.implemented_functions(),
                &catalog,
            );
            score_milestones(&milestones, &suites, &results, &mut report)?;
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
        Cmd::Check { sel } => cmd_check(&root, &sel, None),
        Cmd::UpdateBaseline {
            sel,
            allow_regressions,
        } => cmd_check(&root, &sel, Some(allow_regressions)),
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
