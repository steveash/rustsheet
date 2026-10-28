---
id: T-3008
title: "Recalculation work budget: pass allowance, share mode and ceiling (SPEC-0028 step 1)"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2807]
---

## Goal
One recalculation (`Model::evaluate`) spends a bounded number of work units, and which cells give up past the bound
does not depend on the evaluation order (SPEC-0028 sections 1-3 and 5-6).

## Context
SPEC-0028 (T-2807) is the design. Today only one formula run is bounded (SPEC-0024, `MAX_FORMULA_WORK`), so
1,000 near-limit cells are about 2 hours. Code: crates/rustsheet-engine/src/area_budget.rs (`with_area_budget`,
where the runs are settled), evaluation.rs (`Evaluation`, `evaluate`, `evaluate_passes`, `run_pass`,
`evaluate_formula_cell`'s unwinding), defined_names.rs (`run_cell_formula`). The hot path (`spend_work`) must
not change. The overrun is a new `RecalcBudget` field, not a `Restart` variant: `RestartLog::record` must
learn no fact from it (SPEC-0028 section 2). The share's `4 * N` is saturating and `N` ≥ 1. Sweeps and data tables run under the share in share mode but draw on the ceiling only in T-3009.

## Acceptance criteria
- [x] `MAX_PASS_WORK`, `MAX_RECALC_WORK` and `RecalcBudget` in `Evaluation`, settled in `with_area_budget`
      (SPEC-0028 section 2); `evaluate_formula` and conditional formats draw on no pool.
- [x] Overrun of the committed or thrown-away allowance re-runs the round in share mode; past the ceiling the
      recalculation is exhausted; the two errors and messages (section 3).
- [x] The SPEC-0028 test plan's tests except step 2's, counter-based, including order independence under a
      permuted anchor order and the worked example's values.
- [x] Calibration measured (section 5: heaviest eval pass and recalculation each at most a quarter of their
      number) and the DEVIATION entry of section 6 added to docs/excel-semantics/recalc.md; the doc comments of
      `set_max_evaluation_depth` and `set_max_evaluation_stack` amended.
- [x] No baseline gains a line; the calc bench does not slow by more than 2%.
- [x] `make check` green; reviewers approve.

## Out of scope
- Sweeps and data tables drawing on the ceiling (T-3009).
- Incremental recalculation (SPEC-0028 section 7 is a requirement on SPEC-0002, T-0401).

## Outcome
`recalc_budget.rs` holds `MAX_PASS_WORK` (2^30), `MAX_RECALC_WORK` (2^32) and `RecalcBudget`, which lives in
`Evaluation::recalc` only while `Model::evaluate` runs. `with_area_budget` settles each formula run's spend into the
pools at its start and end (`start_run`, `end_run`); the hot path `spend_work` is unchanged. A run that runs out of a
pool sets an overrun that abandons the pass like a restart (`Evaluation::abandoned`), and `evaluate_passes` handles it
before `RestartLog::record`. Past the allowance the round re-runs in share mode (`TOO_MUCH_RECALC_WORK`); past the
ceiling every formula cell is `#VALUE!` (`RECALC_EXHAUSTED`), and the sweeps and data tables do not run. Runs cut short
by the depth limit, abandoned passes and passes replaced by an Excel-order pass count as thrown away. Tests:
`test_recalc_budget.rs` (13 counter-based tests, including order independence under permuted anchors and moved
formulas, and the worked example scaled by 2^14), plus an ignored release timing of the worked example at the default
numbers (24 to 33 s; recalc.md). Calibration over every suite, calipers, poi and corpus-bench included: the heaviest pass
and recalculation spend 180,967,707 units (probes/t2759-lunar-packed-7101), under a quarter of both numbers. Eval delta:
0 (no regressions, no baseline line).
