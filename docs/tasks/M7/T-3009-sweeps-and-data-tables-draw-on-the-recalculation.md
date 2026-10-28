---
id: T-3009
title: "Sweeps and data tables draw on the recalculation ceiling (SPEC-0028 step 2)"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-3008]
---

## Goal
Iterative sweeps and data-table runs draw on the recalculation ceiling of SPEC-0028 and stop as their count caps do
when it runs out (SPEC-0028 section 4).

## Context
After T-3008 a round of passes is bounded, but the sweeps (iteration.rs `sweep`, `MAX_SWEEP_EVALUATIONS` = 2^25
runs) and the data tables (data_tables.rs, `MAX_DATA_TABLE_EVALUATIONS` = 2^25 runs) still multiply the
per-formula budget by their run counts. Their order is canonical (row-major; storage order), so stopping at the
ceiling is deterministic.

## Acceptance criteria
- [x] Each sweep member run and each data-table run draws on `MAX_RECALC_WORK` and not on the pass allowance.
- [x] Past the ceiling the sweeps stop where they are (the member keeps its value from before that run) and the
      current and later data-table result cells are `#NUM!`; a test for each, with test-lowered limits.
- [x] recalc.md "Iterative sweeps evaluate at most 2^25 members per recalculation" and "Data tables re-evaluate the
      workbook at most 2^25 formulas per recalculation" each gain the sentence of SPEC-0028 section 6; the
      SPEC-0028 DEVIATION entry drops its "until T-3009" note.
- [x] No baseline gains a line.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_SWEEP_EVALUATIONS` or `MAX_DATA_TABLE_EVALUATIONS`.

## Outcome
Sweep and data-table runs now draw on `MAX_RECALC_WORK` (recalc_budget.rs: `start_run` pools every
run inside `Model::evaluate`; outside a pass only the ceiling applies and nothing is committed or
thrown away against the pass allowance). Past the ceiling `sweep` stops with the member's value from
before the run, and `evaluate_data_table` turns the current and later results into `#NUM!`
(`spend_all_past_ceiling`); the second round of passes is then exhausted. Tests in
test_recalc_budget.rs (sweeps spend but do not commit; the 11th sweep run past the ceiling; the
third substitution and a header read past the ceiling). recalc.md: the sweep and data-table entries
gain their sentence, the SPEC-0028 entry drops "Until T-3009" and notes that a memo-skipped table
spends nothing. Eval delta: none (no recalculation in eval comes near the ceiling).
