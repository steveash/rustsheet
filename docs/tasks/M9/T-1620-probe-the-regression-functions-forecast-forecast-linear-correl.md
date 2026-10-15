---
id: T-1620
title: "Probe the regression functions FORECAST, FORECAST.LINEAR, CORREL, PEARSON, RSQ, SLOPE, COVAR, STEYX, INTERCEPT, TREND to the evidence floor"
milestone: M9
lane: research
status: todo
priority: P1
size: S
tier: sonnet
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
