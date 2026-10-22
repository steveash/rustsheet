---
id: T-2362
title: "MINVERSE singularity rule: invert residual-pivot matrices, #NUM! only where Excel does"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
MINVERSE matches Excel for every cell of probes/minverse-singular-near-singular, including the gen-m5/9284730052aff46c Calc!A39 cell (1x1 pivot 1E-108).

## Context
T-2067 probed MINVERSE (excel-graph). Excel returns `#NUM!` only for `{1,2;2,4}` (exact zero pivot) and `{1,1;1,1.0000000000000002}` (last LU pivot 2.22E-16); it inverts `{1,2,3;4,5,6;7,8,9}` (last pivot 1.11E-16, entries about 9.0E15), `{1E-20}`, `{1E-300}` (9.999999999999999E+299, not 1E+300) and `{1E-17,0;0,1}`. So the rule is not a monotone absolute-pivot cutoff: the engine's `f64::EPSILON` cutoff in `fn_minverse` (functions/math_and_trigonometry/matrix.rs) is wrong in both directions around it. The oracle values (arrays.md "MINVERSE") give the target; first find which test and which inversion arithmetic reproduce the huge residual entries bit-exactly (the 3x3 entries -4503599627370498, 9007199254740992, ...). Use the excel-probe skill for further probes if the data do not pin it.

## Acceptance criteria
- [ ] Every cell of probes/minverse-singular-near-singular passes; the gen-m5 line leaves eval/baseline/gen-m5.txt and the probe's lines leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
