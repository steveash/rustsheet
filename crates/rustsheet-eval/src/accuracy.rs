//! Gate A, the accuracy gate (SPEC-0029, decision 0188 R2): `rustsheet-eval accuracy`.
//!
//! The reference tables live in `eval/accuracy/` (the `accuracy` suite of
//! [`Suite::load_named`], outside `eval/suites/`): one directory per function under `tables/`,
//! one immutable batch per `<batch>.case.json`, each with an oracle sidecar (Excel's values, once
//! filled) and a `rustsheet-math/v1` sidecar ([`math`]). Every `check` cell of a batch is a point.
//!
//! The check evaluates each batch once through the runner (panic isolation, per-case timeout),
//! classifies every point with [`envelope::envelope`], applies the known-failure list and the R3
//! marks ([`lists`]), and fails on a failing point, a stale known-failure line, an R3 mark without
//! evidence or a table defect.

pub mod envelope;
pub mod lists;
pub mod math;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::Serialize;

use crate::engine::Engine;
use crate::runner::{cell_id, run_case_values, CaseValues, CellKind};
use crate::suite::{CaseFile, Suite, ACCURACY_SUITE};
use crate::value::Value;
use envelope::{envelope, excel_off_math, relative_error, Envelope, MathValue};
use lists::{check_mark, KnownFailures, ListError, R3Marks};
use math::{math_sidecar, Attached, MathTable, Stratum};

/// Where the check writes every point by default, relative to the repository root (git-ignored).
pub const JSON_PATH: &str = "eval/runs/accuracy.json";

/// Why the check could not run at all (as opposed to running and failing).
#[derive(Debug, thiserror::Error)]
pub enum AccuracyError {
    /// The accuracy manifest is missing or invalid, or its cases cannot be listed.
    #[error("accuracy suite: {0}")]
    Suite(String),
    /// A committed list is unreadable or malformed.
    #[error(transparent)]
    List(#[from] ListError),
    /// The JSON output cannot be written.
    #[error("{path}: {message}")]
    Write { path: String, message: String },
}

/// How a point counts for the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Standing {
    /// Inside the envelope.
    Inside,
    /// Outside, and listed in `known-failures.tsv`: reported, not failed.
    KnownFailure,
    /// Outside (or an error mismatch) and not listed: fails the check.
    Failing,
    /// A table defect: fails the check.
    TableDefect,
}

/// One checked point.
#[derive(Debug, Clone, Serialize)]
pub struct Point {
    /// `accuracy/<FUNCTION>/<batch>!<Sheet>!<A1>`.
    pub id: String,
    /// The function, the table's directory name.
    pub table: String,
    pub batch: String,
    /// The formula without its leading `=`.
    pub formula: Option<String>,
    pub stratum: Option<Stratum>,
    /// The engine's value; `None` when the batch did not run.
    pub engine: Option<Value>,
    /// Excel's value; `None` while pending.
    pub excel: Option<Value>,
    pub math: Option<MathValue>,
    pub class: Envelope,
    pub standing: Standing,
    /// Whether `math-over-excel.tsv` marks the point (R3).
    pub r3: bool,
    /// The owner task of its known-failure line, when listed.
    pub owner: Option<String>,
    /// Why a point failed beyond its values: a stale math value, a batch that crashed.
    pub note: Option<String>,
}

impl Point {
    /// How far the engine is from the math, relatively; infinite when there is no such distance
    /// (an error, a missing value), so those sort first.
    #[must_use]
    pub fn rel_math(&self) -> f64 {
        match (&self.engine, &self.math) {
            (Some(Value::Number(e)), Some(MathValue::Number(m))) => relative_error(*e, *m),
            _ => f64::INFINITY,
        }
    }
}

/// A list line that fails the check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListProblem {
    pub file: &'static str,
    pub line: usize,
    pub id: String,
    pub why: String,
}

/// The check's result.
#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub points: Vec<Point>,
    /// Known-failure lines whose point passes or does not exist (stale: remove them).
    pub stale_known_failures: Vec<ListProblem>,
    /// R3 marks citing no MATH-OVER-EXCEL entry, naming no point, or without evidence.
    pub bad_marks: Vec<ListProblem>,
    /// Wall time of the whole check, in seconds.
    pub seconds: f64,
}

/// Runs the check over the accuracy suite of the repository at `root`.
///
/// # Errors
/// When the suite or a list cannot be loaded. A defective batch is a table defect, not an error.
pub fn run(
    root: &Path,
    engine: &Arc<dyn Engine + Send>,
    timeout: Duration,
) -> Result<Check, AccuracyError> {
    let start = Instant::now();
    let suite = Suite::load_named(root, ACCURACY_SUITE)
        .map_err(|e| AccuracyError::Suite(format!("{e:#}")))?;
    let known = lists::load_known_failures(&suite.dir)?;
    let marks = lists::load_r3_marks(&suite.dir)?;
    let cases = suite
        .cases()
        .map_err(|e| AccuracyError::Suite(format!("{e:#}")))?;
    let batches: Vec<Vec<Point>> = cases
        .par_iter()
        .map(|case| {
            let ran = run_case_values(engine, &suite, case, timeout);
            batch_points(case, &ran, &marks)
        })
        .collect();
    let mut points: Vec<Point> = batches.into_iter().flatten().collect();
    let stale_known_failures = apply_known_failures(&mut points, &known);
    let bad_marks = check_marks(root, &points, &marks);
    Ok(Check {
        points,
        stale_known_failures,
        bad_marks,
        seconds: start.elapsed().as_secs_f64(),
    })
}

/// The table (function) and batch of a case id `accuracy/<FUNCTION>/<batch>.case.json`.
fn table_and_batch(case_id: &str) -> Option<(String, String)> {
    let rel = case_id.strip_prefix(&format!("{ACCURACY_SUITE}/"))?;
    let (table, file) = rel.split_once('/')?;
    let batch = file.strip_suffix(".case.json")?;
    (!table.is_empty() && !batch.is_empty() && !batch.contains('/'))
        .then(|| (table.to_string(), batch.to_string()))
}

/// Classifies every point of one evaluated batch (known failures are applied afterwards).
fn batch_points(case: &CaseFile, ran: &CaseValues, marks: &R3Marks) -> Vec<Point> {
    let base = case
        .id
        .strip_suffix(".case.json")
        .unwrap_or(&case.id)
        .to_string();
    let defect = |why: String| Point {
        id: cell_id(&base, "*"),
        table: String::new(),
        batch: String::new(),
        formula: None,
        stratum: None,
        engine: None,
        excel: None,
        math: None,
        class: Envelope::Table,
        standing: Standing::TableDefect,
        r3: false,
        owner: None,
        note: Some(why),
    };
    let Some((table, batch)) = table_and_batch(&case.id) else {
        return vec![defect(
            "a batch must be tables/<FUNCTION>/<batch>.case.json".into(),
        )];
    };
    if ran.check.is_empty() {
        let why = ran
            .result
            .message
            .clone()
            .unwrap_or_else(|| "the batch has no check cells".into());
        return vec![Point {
            table,
            batch,
            ..defect(why)
        }];
    }
    let math = MathTable::load(&math_sidecar(&case.sidecar));
    let excel_cells: BTreeMap<String, _> = ran
        .result
        .cells
        .iter()
        .filter(|c| c.kind == CellKind::Scored && c.from_excel)
        .map(|c| (c.key(), c))
        .collect();
    ran.check
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let key = format!("{}!{}", cell.sheet, cell.addr);
            let id = cell_id(&base, &key);
            let r3 = marks.contains_key(&id);
            let excel = excel_cells.get(&key).copied();
            let mut note = None;
            let (math_cell, stratum) = match &math {
                Ok(Some(t)) => match t.attach(&cell.sheet, cell.addr, cell.formula.as_deref()) {
                    Attached::Value(m) => (Some(m.value.clone()), m.stratum),
                    Attached::Stale(m) => {
                        note = Some(format!("stale math value for ={}", m.formula));
                        (None, m.stratum)
                    }
                    Attached::Missing => {
                        note = Some("no math value".into());
                        (None, None)
                    }
                },
                Ok(None) => {
                    note = Some("the batch has no math sidecar".into());
                    (None, None)
                }
                Err(e) => {
                    note = Some(e.to_string());
                    (None, None)
                }
            };
            let engine = ran.values.get(i).cloned();
            let class = match &engine {
                Some(v) => envelope(excel, v, math_cell.as_ref(), r3),
                None if math_cell.is_none() => Envelope::Table,
                None => {
                    // A batch that crashed or timed out fails every point in it.
                    note = Some(format!(
                        "the batch did not run: {:?} {}",
                        ran.result.status,
                        ran.result.message.as_deref().unwrap_or_default()
                    ));
                    Envelope::Outside
                }
            };
            let standing = match class {
                c if c.is_inside() => Standing::Inside,
                Envelope::Table => Standing::TableDefect,
                _ => Standing::Failing,
            };
            if class == Envelope::Table && note.is_none() {
                note = Some("a math domain error against an Excel number".into());
            }
            Point {
                id,
                table: table.clone(),
                batch: batch.clone(),
                formula: cell.formula.clone(),
                stratum,
                engine,
                excel: excel.map(|c| c.expected.clone()),
                math: math_cell,
                class,
                standing,
                r3,
                owner: None,
                note,
            }
        })
        .collect()
}

/// Marks listed failing points as known failures, and returns the stale lines: a listed point
/// that passes, or that no table holds.
fn apply_known_failures(points: &mut [Point], known: &KnownFailures) -> Vec<ListProblem> {
    let mut seen = BTreeSet::new();
    let mut stale = Vec::new();
    for p in points.iter_mut() {
        let Some(line) = known.get(&p.id) else {
            continue;
        };
        seen.insert(p.id.clone());
        p.owner = Some(line.owner.clone());
        match p.standing {
            Standing::Failing => p.standing = Standing::KnownFailure,
            Standing::Inside => stale.push(ListProblem {
                file: lists::KNOWN_FAILURES,
                line: line.line,
                id: p.id.clone(),
                why: "the point is inside the envelope: remove the line".into(),
            }),
            Standing::KnownFailure | Standing::TableDefect => {}
        }
    }
    for (id, line) in known.iter().filter(|(id, _)| !seen.contains(*id)) {
        stale.push(ListProblem {
            file: lists::KNOWN_FAILURES,
            line: line.line,
            id: id.clone(),
            why: "no table holds the point".into(),
        });
    }
    stale.sort_by_key(|p| p.line);
    stale
}

/// The R3 marks that fail: an entry that is not MATH-OVER-EXCEL, a point id no table holds, or
/// a filled marked point where Excel is not off the math by more than G (decision 0188 R3).
/// Lines naming eval cells are T-3359's and T-3380's; here only their entries are checked.
fn check_marks(root: &Path, points: &[Point], marks: &R3Marks) -> Vec<ListProblem> {
    let by_id: BTreeMap<&str, &Point> = points.iter().map(|p| (p.id.as_str(), p)).collect();
    let mut bad = Vec::new();
    for (id, mark) in marks {
        let problem = |why: String| ListProblem {
            file: lists::MATH_OVER_EXCEL,
            line: mark.line,
            id: id.clone(),
            why,
        };
        if let Err(e) = check_mark(mark, root) {
            bad.push(problem(e.to_string()));
            continue;
        }
        if !id.starts_with(&format!("{ACCURACY_SUITE}/")) {
            continue;
        }
        let Some(point) = by_id.get(id.as_str()) else {
            bad.push(problem("no table holds the point".into()));
            continue;
        };
        if let (Some(excel), Some(math)) = (&point.excel, &point.math) {
            if !excel_off_math(excel, math) {
                bad.push(problem(format!(
                    "no evidence: Excel's {} is not off the math by more than G = 1e-4",
                    show(Some(excel))
                )));
            }
        } else if point.excel.is_some() {
            bad.push(problem("no evidence: the point has no math value".into()));
        }
    }
    bad.sort_by_key(|p| p.line);
    bad
}

/// A value as the output shows it.
fn show(v: Option<&Value>) -> String {
    match v {
        None => "pending".into(),
        Some(Value::Number(n)) => format!("{n:E}"),
        Some(other) => other.to_string(),
    }
}

fn show_math(m: Option<&MathValue>) -> String {
    match m {
        None => "none".into(),
        Some(MathValue::Number(n)) => format!("{n:E}"),
        Some(MathValue::DomainError(why)) => format!("domain error ({why})"),
    }
}

impl Check {
    /// Whether the gate passes: no failing point, stale known-failure line, bad R3 mark or table
    /// defect.
    #[must_use]
    pub fn passes(&self) -> bool {
        self.stale_known_failures.is_empty()
            && self.bad_marks.is_empty()
            && self
                .points
                .iter()
                .all(|p| matches!(p.standing, Standing::Inside | Standing::KnownFailure))
    }

    fn count(&self, f: impl Fn(&Point) -> bool) -> usize {
        self.points.iter().filter(|p| f(p)).count()
    }

    /// The summary line, then every failure worst first by relative error against the math, the
    /// known failures, the stale lines, the bad marks and the table defects.
    #[must_use]
    pub fn render(&self) -> String {
        let tables: BTreeSet<&str> = self
            .points
            .iter()
            .filter(|p| !p.table.is_empty())
            .map(|p| p.table.as_str())
            .collect();
        let mut owners: BTreeMap<&str, usize> = BTreeMap::new();
        for p in &self.points {
            if let (Standing::KnownFailure, Some(o)) = (p.standing, &p.owner) {
                *owners.entry(o.as_str()).or_default() += 1;
            }
        }
        let owners: Vec<String> = owners.iter().map(|(o, n)| format!("{o} {n}")).collect();
        let known = self.count(|p| p.standing == Standing::KnownFailure);
        let mut out = format!(
            "accuracy: {} points in {} tables (pending {}): {} in envelope (Excel 15 digits only {}, math only {}), {} known failures{}, {} failing, {} table defects; missing tables n/a (T-3379); {:.2} s\n",
            self.points.len(),
            tables.len(),
            self.count(|p| p.excel.is_none()),
            self.count(|p| p.standing == Standing::Inside),
            self.count(|p| p.class == Envelope::Excel15),
            self.count(|p| p.class == Envelope::Math),
            known,
            if owners.is_empty() {
                String::new()
            } else {
                format!(" ({})", owners.join(", "))
            },
            self.count(|p| p.standing == Standing::Failing),
            self.count(|p| p.standing == Standing::TableDefect),
            self.seconds,
        );
        let mut listed: Vec<&Point> = self
            .points
            .iter()
            .filter(|p| matches!(p.standing, Standing::Failing | Standing::KnownFailure))
            .collect();
        // Failing points first, then known failures; each worst first.
        listed.sort_by(|a, b| {
            (a.standing == Standing::KnownFailure)
                .cmp(&(b.standing == Standing::KnownFailure))
                .then(b.rel_math().total_cmp(&a.rel_math()))
                .then(a.id.cmp(&b.id))
        });
        for p in listed {
            let tag = match (&p.standing, &p.owner) {
                (Standing::KnownFailure, Some(o)) => format!("KNOWN ({o})"),
                _ => "FAIL".into(),
            };
            let _ = writeln!(
                out,
                "{tag} {} {} ={}{}\n  engine {}  excel {}  math {}  rel(math) {:.1E}{}",
                p.id,
                p.stratum.map_or("-", Stratum::name),
                p.formula.as_deref().unwrap_or(""),
                if p.class == Envelope::ErrorMismatch {
                    "  [error mismatch]"
                } else {
                    ""
                },
                show(p.engine.as_ref()),
                show(p.excel.as_ref()),
                show_math(p.math.as_ref()),
                p.rel_math(),
                p.note
                    .as_deref()
                    .map(|n| format!("  ({n})"))
                    .unwrap_or_default(),
            );
        }
        for s in &self.stale_known_failures {
            let _ = writeln!(out, "STALE {} line {}: {}: {}", s.file, s.line, s.id, s.why);
        }
        for m in &self.bad_marks {
            let _ = writeln!(
                out,
                "BAD MARK {} line {}: {}: {}",
                m.file, m.line, m.id, m.why
            );
        }
        for p in self
            .points
            .iter()
            .filter(|p| p.standing == Standing::TableDefect)
        {
            let _ = writeln!(
                out,
                "TABLE {} ={}: {}",
                p.id,
                p.formula.as_deref().unwrap_or(""),
                p.note.as_deref().unwrap_or("table defect")
            );
        }
        out
    }

    /// Writes every point's class, values and table to `path` (T-3359 reads it).
    ///
    /// # Errors
    /// When the file cannot be written.
    pub fn write_json(&self, path: &Path) -> Result<(), AccuracyError> {
        let fail = |e: &dyn std::fmt::Display| AccuracyError::Write {
            path: path.display().to_string(),
            message: e.to_string(),
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| fail(&e))?;
        }
        let text = serde_json::to_string(&serde_json::json!({
            "passes": self.passes(),
            "check": self,
        }))
        .map_err(|e| fail(&e))?;
        std::fs::write(path, text).map_err(|e| fail(&e))
    }
}

#[cfg(test)]
mod tests;
