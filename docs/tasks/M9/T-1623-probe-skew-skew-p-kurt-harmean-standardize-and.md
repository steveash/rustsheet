---
id: T-1623
title: "Probe SKEW, SKEW.P, KURT, HARMEAN, STANDARDIZE and SUMX2PY2, SUMXMY2, SUMX2MY2, MDETERM, ECMA.CEILING to the evidence floor"
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
T-0056): SKEW (14), SKEW.P (10), KURT (14), HARMEAN (13), STANDARDIZE (11), SUMX2PY2 (11), SUMXMY2 (13), SUMX2MY2 (14), MDETERM (17), ECMA.CEILING (11) cells today (`make eval`, report "Below the evidence floor", 2026-10-15).

## Context
docs/decisions/0062 (evidence-floor ratchet); `excel-probe` skill. Write one or more spec probe
cases under eval/suites/probes/cases/, fill them with the oracle, and cover every parameter mode
of each function (catalog signatures in eval/catalog/functions.yaml) and the catalog's edge
pools (tools/src/rustsheet_tools/gen/generator.py "Edge-value pools"). Edge cases to include:
too few points and zero deviation (#DIV/0!), non-positive values for HARMEAN, standard_dev <= 0, mismatched array sizes for the SUMX* family and blanks/text pairs, non-square and singular matrices and text in MDETERM, ECMA.CEILING with negative numbers and significance signs. Record every quirk found in docs/excel-semantics/ and file a fix task for each mismatch
(`make new-task`); fixing the engine is out of scope.

## Acceptance criteria
- [ ] Each listed function has ≥ 20 Excel-evidence cells in the report.
- [ ] `make eval-baseline` lowers quality/evidence-floor.json by the number of functions lifted
  over the floor; new failing cells are recorded as new-coverage failures with fix tasks.
- [ ] Quirks recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes for the mismatches found (filed as tasks).
