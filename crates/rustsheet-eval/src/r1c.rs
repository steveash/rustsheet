//! Decision 0188 R1(c): which visibly-off cells a milestone's probe row and ironcalc-calc row
//! count, and which are Phase 1b evidence instead (T-3427).
//!
//! A row counts a visibly-off cell (decision 0083, [`Visible::is_off`]) unless:
//! - an open M14 task alone owns it (R7, decision 0143's generated list,
//!   [`Phase1Exceptions::desktop_owns`]): it leaves the row ([`R1c::M14Owned`]);
//! - `math-over-excel.tsv` marks it and the engine returns the math within ε (R3): it passes
//!   ([`R1c::R3Pass`]);
//! - it is a number inside Gate A's envelope ([`envelope`]: within ε of its 50-digit math value,
//!   T-3380) and Excel's value is within G of that math: it is Phase 1b evidence
//!   ([`R1c::Phase1b`]), measured and reported every run but not counted.
//!
//! Everything else counts ([`R1c::Counts`]): a type, error or text cell; a number outside the
//! envelope, or with no math value (decision 0188 R1(c): until a cell has a math reference it
//! counts as failing the envelope); and a number within ε of the math where Excel is more than
//! G = 1e-4 off it without an R3 mark. That last case is R1(c)'s "wrong number from a semantic
//! rule": Excel applies a rule the math does not, R3 says such a gap needs an owner ruling, and
//! until one exists the engine is held to Excel's value.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::accuracy::envelope::{envelope, excel_off_math, MathValue};
use crate::accuracy::lists::R3Marks;
use crate::closeness::Visible;
use crate::phase1_exceptions::Phase1Exceptions;
use crate::runner::{cell_id, CaseResult, CellResult};
use crate::value::{Value, Verdict};

/// How R1(c) reads one visibly-off cell (see the module doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum R1c {
    /// The row counts it.
    Counts,
    /// Inside Gate A's envelope with Excel within G of the math: Phase 1b evidence.
    Phase1b,
    /// An R3 cell where the engine returns the math: passing.
    R3Pass,
    /// Only open M14 tasks own it (R7): it leaves the row.
    M14Owned,
}

/// Classifies cell `c` of case `case`, a visibly-off cell (`None` for any other cell).
#[must_use]
pub fn classify(
    case: &str,
    c: &CellResult,
    r3: &R3Marks,
    exceptions: &Phase1Exceptions,
) -> Option<R1c> {
    if !(c.verdict.is_failure() && c.verdict != Verdict::Deviation && Visible::is_off(c)) {
        return None;
    }
    let key = c.key();
    if exceptions.desktop_owns(case, &key) {
        return Some(R1c::M14Owned);
    }
    let (Value::Number(_), Value::Number(_)) = (&c.expected, &c.actual) else {
        return Some(R1c::Counts);
    };
    let marked = r3.contains_key(&cell_id(case, &key));
    if !envelope(Some(c), &c.actual, c.math.as_ref(), marked).is_inside() {
        return Some(R1c::Counts);
    }
    if marked {
        return Some(R1c::R3Pass);
    }
    let semantic = c
        .math
        .as_ref()
        .is_some_and(|m| matches!(m, MathValue::Number(_)) && excel_off_math(&c.expected, m));
    Some(if semantic { R1c::Counts } else { R1c::Phase1b })
}

/// Every visibly-off cell of a run with its [`R1c`] class, by `case!Sheet!A1`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct R1cCells {
    cells: BTreeMap<String, R1c>,
}

impl R1cCells {
    /// Classifies every visibly-off cell of `results`.
    #[must_use]
    pub fn build(results: &[CaseResult], r3: &R3Marks, exceptions: &Phase1Exceptions) -> Self {
        let mut cells = BTreeMap::new();
        for r in results {
            for c in &r.cells {
                if let Some(class) = classify(&r.id, c, r3, exceptions) {
                    cells.insert(cell_id(&r.id, &c.key()), class);
                }
            }
        }
        Self { cells }
    }

    /// Classes given directly, by `case!Sheet!A1`, for tests of the readers.
    #[cfg(test)]
    pub(crate) fn from_classes<'a>(cells: impl IntoIterator<Item = (&'a str, R1c)>) -> Self {
        Self {
            cells: cells
                .into_iter()
                .map(|(id, c)| (id.to_string(), c))
                .collect(),
        }
    }

    /// The class of a visibly-off cell by its `case!Sheet!A1`; [`R1c::Counts`] for a cell the run
    /// did not classify, so nothing reads as met early.
    #[must_use]
    pub fn class(&self, id: &str) -> R1c {
        self.cells.get(id).copied().unwrap_or(R1c::Counts)
    }

    /// Whether R1(c) counts the visibly-off cell `id`.
    #[must_use]
    pub fn counts(&self, id: &str) -> bool {
        self.class(id) == R1c::Counts
    }

    /// The Phase 1b evidence of the run: per suite, the visibly-off numeric cells inside Gate A's
    /// envelope that the probe rows leave out.
    #[must_use]
    pub fn phase1b(&self) -> Phase1bEvidence {
        let mut out = Phase1bEvidence::default();
        for (id, class) in &self.cells {
            let suite = id.split('/').next().unwrap_or_default();
            if !EVIDENCE_SUITES.contains(&suite) {
                continue;
            }
            let row = out.suites.entry(suite.to_string()).or_default();
            match class {
                R1c::Counts => row.counts += 1,
                R1c::Phase1b => row.phase1b += 1,
                R1c::R3Pass => row.r3_pass += 1,
                R1c::M14Owned => row.m14_owned += 1,
            }
        }
        out
    }
}

/// The suites whose visibly-off numeric cells R1(c) reads as Phase 1b evidence.
pub const EVIDENCE_SUITES: [&str; 3] = ["probes", "probes-numeric", "ironcalc-calc"];

/// One suite's visibly-off cells by [`R1c`] class.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct R1cCounts {
    pub counts: usize,
    pub phase1b: usize,
    pub r3_pass: usize,
    pub m14_owned: usize,
}

/// R1(c)'s classes per [`EVIDENCE_SUITES`] suite the run scored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase1bEvidence {
    pub suites: BTreeMap<String, R1cCounts>,
}

impl Phase1bEvidence {
    /// The report line: per suite, the counted cells, the Phase 1b evidence, the R3 passes and
    /// the M14-owned cells; nothing when no evidence suite ran.
    #[must_use]
    pub fn line(&self) -> Option<String> {
        if self.suites.is_empty() {
            return None;
        }
        let parts: Vec<String> = self
            .suites
            .iter()
            .map(|(suite, c)| {
                format!(
                    "{suite} {} counted, {} Phase 1b evidence, {} R3 passing, {} M14-owned",
                    c.counts, c.phase1b, c.r3_pass, c.m14_owned
                )
            })
            .collect();
        Some(format!(
            "Visibly-off cells under decision 0188 R1(c): {}. Phase 1b evidence is a number inside Gate A's envelope (within 1e-9 of its 50-digit math value) whose Excel value is within 1e-4 of the math; a number with no math value counts.",
            parts.join("; ")
        ))
    }
}

#[cfg(test)]
mod tests;
