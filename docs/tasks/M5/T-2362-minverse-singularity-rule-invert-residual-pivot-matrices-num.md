---
id: T-2362
title: "MINVERSE singularity rule: invert residual-pivot matrices, #NUM! only where Excel does"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
MINVERSE matches Excel for every cell of probes/minverse-singular-near-singular, including the gen-m5/9284730052aff46c Calc!A39 cell (1x1 pivot 1E-108).

## Context
T-2067 probed MINVERSE (excel-graph). Excel returns `#NUM!` only for `{1,2;2,4}` (exact zero pivot) and `{1,1;1,1.0000000000000002}` (last LU pivot 2.22E-16); it inverts `{1,2,3;4,5,6;7,8,9}` (last pivot 1.11E-16, entries about 9.0E15), `{1E-20}`, `{1E-300}` (9.999999999999999E+299, not 1E+300) and `{1E-17,0;0,1}`. So the rule is not a monotone absolute-pivot cutoff: the engine's `f64::EPSILON` cutoff in `fn_minverse` (functions/math_and_trigonometry/matrix.rs) is wrong in both directions around it. The oracle values (arrays.md "MINVERSE") give the target; first find which test and which inversion arithmetic reproduce the huge residual entries bit-exactly (the 3x3 entries -4503599627370498, 9007199254740992, ...). Use the excel-probe skill for further probes if the data do not pin it.

gen-m5 round-2 triage (T-2252, excel-graph oracle, 2026-10-22): one more 1x1 cell, a pivot of 9.3E-25 that Excel inverts with the
plain IEEE reciprocal (arrays.md "MINVERSE: singularity rule", sentence added by this triage).

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/12a3d13f2a5aa246 Calc!A258 | `=MINVERSE(((Data!E1+Data!E1)/TRUNC(2^31)))` (E1 = 1E-15) | 1.0737418239999999e+24 | #NUM! |

## Acceptance criteria
- [ ] Every cell of probes/minverse-singular-near-singular passes; the 2 gen-m5 lines (9284730052aff46c A39, 12a3d13f2a5aa246 A258) leave eval/baseline/gen-m5.txt and the probe's lines leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MDETERM, which already matches Excel on every probed cell.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.
