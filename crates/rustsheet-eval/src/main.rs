//! `rustsheet-eval` — the Excel-compatibility harness CLI.
//!
//! ```text
//! rustsheet-eval suites                         # list suites
//! rustsheet-eval run [--suite S] [--filter X]   # run + write eval/runs/latest/{report.json,report.md}
//! rustsheet-eval check                          # gate: fail on any baseline regression
//! rustsheet-eval update-baseline                # ratchet baselines up (refuses regressions)
//! rustsheet-eval show <case-id> [--all]         # per-cell detail for one case
//! rustsheet-eval snapshot <file.xlsx>           # dump stored formulas/values as JSON
//! ```
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use rustsheet_eval::baseline::{self, Baseline};
use rustsheet_eval::engine::{Engine, RustSheetEngine};
use rustsheet_eval::fetch::fetch_suite;
use rustsheet_eval::report::RunReport;
use rustsheet_eval::runner::{run_suites, CaseResult, RunOptions};
use rustsheet_eval::suite::{BaselineMode, FetchState, Suite};
use rustsheet_eval::xlsx_snapshot::read_snapshot;

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

fn print_summary(report: &RunReport) {
    println!(
        "{:<28} {:>6} {:>9} {:>9} {:>8}",
        "suite", "cases", "cells", "passed", "pass%"
    );
    for s in &report.suites {
        println!(
            "{:<28} {:>6} {:>9} {:>9} {:>7.2}%",
            s.name,
            s.cases,
            s.cells,
            s.cells_passed,
            100.0 * s.cell_pass_rate()
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
    let results = execute(&suites, sel);
    let report = RunReport::build(RustSheetEngine.name(), &results);
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
            "[{name}] known failures {} -> {}  (fixed {}, new-coverage failures {}, REGRESSIONS {}, removed cases {})",
            old.failing.len(),
            current.failing.len(),
            delta.fixed.len(),
            delta.new_failures.len(),
            delta.regressions.len(),
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
        "{}  status={:?}  checked={} passed={}  source={}",
        r.id, r.status, r.checked, r.passed, r.expected_source
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

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    if let Cmd::Snapshot { path } = &cli.cmd {
        let bytes = std::fs::read(path).with_context(|| path.display().to_string())?;
        let snapshot = read_snapshot(&bytes)?;
        // Tools decide "would the suite trust these cached values?" from this flag rather than
        // re-implementing the rule (CLAUDE.md: tools never re-implement xlsx value parsing).
        let mut json = serde_json::to_value(&snapshot)?;
        if let Some(obj) = json.as_object_mut() {
            obj.insert(
                "excel_authored".into(),
                serde_json::Value::Bool(snapshot.is_excel_authored()),
            );
        }
        println!("{}", serde_json::to_string_pretty(&json)?);
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
            let results = execute(&suites, &sel);
            let report = RunReport::build(RustSheetEngine.name(), &results);
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
        Cmd::Snapshot { .. } => unreachable!("handled above"),
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
