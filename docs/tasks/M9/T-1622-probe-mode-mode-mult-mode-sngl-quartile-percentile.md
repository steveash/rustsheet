---
id: T-1622
title: "Probe MODE, MODE.MULT, MODE.SNGL, QUARTILE*, PERCENTILE, PERCENTRANK, AVEDEV and DEVSQ to the evidence floor"
milestone: M9
lane: research
status: done
priority: P1
size: S
tier: sonnet
depends_on: [T-0056]
---

## Goal
Bring each of these in-scope functions to at least 20 Excel-evidence cells (the evidence floor,
T-0056): MODE (8), MODE.MULT (13), MODE.SNGL (16), QUARTILE (9), QUARTILE.INC (18), QUARTILE.EXC (19), PERCENTILE (11), PERCENTRANK (12), AVEDEV (10), DEVSQ (16) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
no repeated value (#N/A), ties and their order, MODE.MULT spill shape for row vs. column input, quart 0-4 and fractional/out-of-range quart, EXC's narrower valid range, k at 0/1 and between data points, PERCENTRANK's significance argument and x outside the data, text/booleans in ranges vs. as arguments. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
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
Added probes/stat-order-floor-batch (300 cells, excel-graph). MODE, MODE.SNGL, MODE.MULT, QUARTILE,
QUARTILE.INC/EXC, PERCENTILE, PERCENTRANK, AVEDEV and DEVSQ are now over the floor: below-floor 39 → 34.
11 new failing cells recorded; fix tasks T-1748 (empty arg counted as 0, typed-boolean QUARTILE) and T-1749
(DEVSQ accumulation). Quirks in docs/excel-semantics/statistics.md.
