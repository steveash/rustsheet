---
id: T-1621
title: "Probe LINEST, LOGEST, GROWTH and the FORECAST.ETS family to the evidence floor"
milestone: M9
lane: research
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): LINEST (9), LOGEST (8), GROWTH (16), FORECAST.ETS.SEASONALITY (10), FORECAST.ETS.STAT (14), FORECAST.ETS.CONFINT (19) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
LINEST/LOGEST const and stats flags (the 5x2 stats block, #N/A fill), collinear and multi-column x's, non-positive y's for LOGEST/GROWTH, ETS seasonality 0/1/auto, data_completion and aggregation modes, statistic_type 1-8, confidence_level bounds, timelines with gaps or duplicates. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
(`make new-task`); fixing the engine is out of scope.

## Acceptance criteria
- [ ] Each listed function has ≥ 20 Excel-evidence cells in the report.
- [ ] `make eval-baseline` lowers quality/evidence-floor.json by the number of functions lifted
  over the floor; new failing cells are recorded as new-coverage failures with fix tasks.
- [ ] Quirks recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the mismatches found (filed as tasks).
