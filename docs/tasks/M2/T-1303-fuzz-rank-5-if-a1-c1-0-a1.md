---
id: T-1303
title: "Fuzz: RANK(5,IF(A1:C1>0,A1:XFE1)) reaches a scalar-context cell with a 3x1 array (debug_assert panic)"
milestone: M2
lane: functions-stat
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Evaluating `=RANK(5,IF(A1:C1>0,A1:XFE1))` (and the fuzz input it was reduced from) in a cell never
panics, in debug or release builds, and gives Excel's result.

## Context
Nightly fuzz run 38022063246 (2026-10-10, attempt 2, `main` 3d755e8), target `formula_eval`,
artifact `crash-40c023f20495bdc2301467fa73bc8729b76c317b`. The input is the 50 bytes
`RANK(5,IF(A1:C1>0,A1:CWWWWWWWWWWWWWWWWWWWWWWWWW1))` (base64
`UkFOSyg1LElGKEExOkMxPjAsQTE6Q1dXV1dXV1dXV1dXV1dXV1dXV1dXV1dXV1cxKSk=`); the target sets it as
`=...` in `C3` of its small model (`fuzz/fuzz_targets/formula_eval.rs`). Reproduced by the steward
on 32dec3cb with a debug test through the facade (`Workbook::set`, then `Workbook::value`):

    panicked at crates/rustsheet-engine/src/model.rs:988:25:
    Larger-than-1x1 array reached scalar-context cell (sheet=0, row=3, column=3, 3x1);
    implicit intersection was expected to collapse it.

Stack: `Model::set_cells_with_result` <- `run_top` <- `evaluate_formula_cell`.
Same assert as T-0278 and T-0296 (AGGREGATE), but reached through RANK, so T-0296's fix does
not cover it. The steward narrowed the trigger:
- `=RANK(5,IF(A1:C1>0,A1:XFE1))` and `...A1:CWWWW1)` panic. The second range end is not a valid
  column, so it is a name, and `A1:<undefined name>` is the operand.
- `=RANK(5,IF(A1:C1>0,A1:XFD1))` is `#SPILL!`. `=RANK(5,IF(A1:C1>0,Foo))`,
  `=RANK(5,IF(A1:C1>0,A1:C1))` and `=RANK(5,IF(TRUE,A1:CWWWW1))` are scalars.
Likely source: `numbers_from_rank_ref_arg` (`statistical/chisq.rs`) returns
`Err(CalcResult::Array(..))` for a non-reference array `ref` (T-1208/T-1215, "RANK of a
non-reference array"), and `fn_rank_eq` returns it as is, while static analysis treats this call
as scalar. Make static analysis and evaluation agree on the shape. The existing probes
(probes/rank-nonref-*) may already give Excel's value; if not, use the excel-probe skill.

## Acceptance criteria
- [x] The reproducer is a unit test (decision 0035) that passes in a debug build.
- [x] Static analysis and evaluation agree on RANK/RANK.EQ/RANK.AVG result shapes with a non-reference array `ref`; the value matches excel-graph evidence, or the case is left pending and the task blocked.
- [x] `make check` green; reviewers approve.

## Out of scope
- AGGREGATE (T-0296).

## Outcome
- `static_analysis_rank` now returns `Unknown` (a dynamic-array formula) when RANK's `ref` has an unknown static shape
  (`A1:<name>` inside IF, INDEX, CHOOSECOLS, LET, a defined name). A reference still evaluates to 1x1; an array spills.
  The fuzz reproducer and `RANK(5,INDEX(IF(..),1,0))`, `RANK(5,CHOOSECOLS(..))`, `RANK(5,LET(..))`, which hit the same
  debug assertion, are unit-tested.
- New probes `rank-unknown-shape-ref` and `rank-nonref-error-elements` (excel-graph): an error element of a non-reference
  array ref keeps its error (we gave `#VALUE!`); fixed in `numbers_from_ranked_arg`.
- Eval delta: probes +2 cases, 48 cells, all pass; known failures 914 → 914; regressions 0.
