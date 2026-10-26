---
id: T-2766
title: "The ERROR.TYPE/CELL mark of the paired functions passes through LET, IF, CHOOSE, IFERROR, a LAMBDA call and INDEX (probes/t2576, 8 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The rest of probes/t2576-error-type-cell-paired passes: a number from ERROR.TYPE or `CELL("col"|"row",..)` keeps its
mark through LET, IF, CHOOSE, IFERROR, a LAMBDA call and INDEX, and `ERROR.TYPE({#REF!})` carries it per element, as in Excel.

## Context
Follow-up of T-2576, which marks the number only when the argument is the ERROR.TYPE or CELL call itself (parentheses
aside; `is_marked_number` in crates/rustsheet-engine/src/functions/statistical/paired.rs, `TODO(T-2766)`). Rule and
evidence: statistics.md "ERROR.TYPE and CELL results are #NUM! in the paired functions" (excel-graph, 2026-10-26).
Known failures left in eval/baseline/probes.txt for probes/t2576-error-type-cell-paired:
- Calc!D1 `FTEST({1;2;3},LET(x,ERROR.TYPE(#REF!),x))`, D2 `IF(TRUE,..)`, D3 `CHOOSE(1,..)`, D5 `IFERROR(..,0)`,
  D19 `LAMBDA(x,x)(..)`, D20 `INDEX(..,1)`, D24 `ERROR.TYPE({#REF!})`: Excel #NUM!, ours #DIV/0!.
- Calc!E16 `SUMXMY2(ERROR.TYPE(#REF!),5)`: Excel #NUM!, ours 1 (SUMXMY2 reads through `sumx_arg`, not
  `fn_get_two_matrices`; SUMX2MY2 and SUMX2PY2 are unprobed).
- Calc!E34 `LINEST(ERROR.TYPE(#REF!))`: Excel #VALUE! where `LINEST(2)` is 0 (gen-m9 8d3588d8df0812df A186), ours a fit,
  and its spill F34.
- Calc!D27 `FTEST({1;2;3},SINGLE(ERROR.TYPE(#REF!)))`: Excel #DIV/0!, ours #NAME? (SINGLE is not parsed; move it to the
  task that owns SINGLE if one exists).
- probes/t2577-indirect-ref-array (T-2577; an OFFSET or INDIRECT lifted into an array of references carries the same mark,
  statistics.md "An array of references is #NUM! in the paired functions"): Calc!A64
  `F.TEST(LET(x,INDIRECT("B1",{1}),x),{-1,0,1})` Excel #NUM!, ours #VALUE! (`Model::evaluate_sample_arg` follows only a
  direct OFFSET or INDIRECT, `TODO(T-2766)`); Calc!A148 `SUMXMY2(INDIRECT("B1",{1}),4)` Excel #NUM!, ours #VALUE!.
- Also probe, for an array of references (T-2577), the first argument of RSQ and STEYX, T.TEST's second sample, and
  COVARIANCE.S's and COVAR's second argument, which the engine marks by analogy.
- Also probe the UNVERIFIED positions statistics.md lists (the second argument of COVAR, COVARIANCE.S and the regression
  family; a marked first argument against an error element of the second).
Carrying the mark through bindings needs a design (a value flag, not syntax); split if bigger than S.

## Acceptance criteria
- [ ] The cells above leave eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines.
- [ ] statistics.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
