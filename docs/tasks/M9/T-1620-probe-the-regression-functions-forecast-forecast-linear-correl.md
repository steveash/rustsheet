---
id: T-1620
title: "Probe the regression functions FORECAST, FORECAST.LINEAR, CORREL, PEARSON, RSQ, SLOPE, COVAR, STEYX, INTERCEPT, TREND to the evidence floor"
milestone: M9
lane: research
status: in-progress
priority: P1
size: S
tier: opus
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): FORECAST (5), FORECAST.LINEAR (10), CORREL (6), PEARSON (6), RSQ (6), SLOPE (6), COVAR (7), STEYX (7), INTERCEPT (8), TREND (11) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
unequal lengths (#N/A), fewer than 2 or 3 points (#DIV/0!), zero variance, text/booleans/blanks inside ranges vs. typed as arguments, errors in either array, TREND's new_x's and const, multi-column x's for TREND. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
(`make new-task`); fixing the engine is out of scope.

## Acceptance criteria
- [ ] Each listed function has ≥ 20 Excel-evidence cells in the report.
- [ ] `make eval-baseline` lowers quality/evidence-floor.json by the number of functions lifted
  over the floor; new failing cells are recorded as new-coverage failures with fix tasks.
- [ ] Quirks recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the mismatches found (filed as tasks).

## Outcome
Four probe cases (regression-pairs-a, regression-pairs-b, regression-forecast, regression-trend; 446 formula cells, excel-graph 2026-10-16) lift FORECAST, FORECAST.LINEAR, CORREL, PEARSON, RSQ, SLOPE, COVAR, STEYX, INTERCEPT and TREND over the evidence floor (quality/evidence-floor.json below_floor 52 -> 42). 202 cells fail in ours and are recorded as new-coverage failures (probes known failures 2289 -> 2491, 0 regressions); fix tasks T-1723 (pairing errors, large offsets), T-1724 (COVAR), T-1725 (FORECAST) and T-1726 (TREND). Quirks in docs/excel-semantics/statistics.md.

## Handoff (resumed)
Escalated by a sonnet run (work-loop step 5b: the excel-fidelity reviewer requested changes twice).
- Done: four probe cases with excel-graph sidecars, baseline and evidence floor (52 -> 42), the quirk record in docs/excel-semantics/statistics.md, fix tasks T-1723..T-1726, all on this branch; `make check` passed before the last doc edits.
- Round 1 findings (all addressed): quirk entry format, stale "unprobed" notes, overlap of the fix tasks with T-1292/T-1707/T-1401/T-1290/T-1402, unrecorded quirks. Round 2 findings (applied in the last commit, not yet re-reviewed): STEYX large-offset call attribution, RSQ examples, blank/row-shaped TREND new_x, COVAR single pair wording.
- Next: re-run `make check` and the four reviewers on the current head, fix what they raise, mark the task `status: done` with the Outcome note below, open/update the PR and merge under the steward policy.
- Outcome to restore: Four probe cases (regression-pairs-a, regression-pairs-b, regression-forecast, regression-trend; 446 formula cells) lift FORECAST, FORECAST.LINEAR, CORREL, PEARSON, RSQ, SLOPE, COVAR, STEYX, INTERCEPT and TREND over the evidence floor (below_floor 52 -> 42); 202 cells are new-coverage failures (probes 2289 -> 2491, 0 regressions), fix tasks T-1723..T-1726.
