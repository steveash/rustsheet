---
id: T-2499
title: "A called TAKE, DROP, TRIMRANGE or XLOOKUP result is a reference: TAKE(A1:A3,1)(2) is #REF!"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A called TAKE, DROP, TRIMRANGE or XLOOKUP result is the reference it returns, so the call is
`#REF!` as in Excel: `=TAKE(A1:A3,1)(2)`, `=DROP(A1:A3,1)(2)`, `=TRIMRANGE(A1:A3)(2)` and
`=XLOOKUP(1,A1:A3,A1:A3)(2)` (ours spill or give `#NAME?`).

## Context
T-2405 chained calls after the built-ins `is_reference_operand` accepts. These seven cells (TAKE B1, DROP B1:B2, TRIMRANGE B1:B3, XLOOKUP B1) are new
failures in eval/baseline/probes.txt (probes/chained-calls-builtin-take, -drop, -trimrange, -xlookup, excel-graph
2026-10-23): ours `#NAME?`, because `LambdaCallKind`'s callee is read by
`evaluate_reference_arg` (functions/util.rs). There an unlifted TAKE gives its value
(`reference_producer`) and XLOOKUP is not a reference case at all, so the call sees an array or a
value and calls it like a LET-bound value. Read them as references in the callee
(`call_callee_as_reference` / node_frames.rs `LambdaCallKind`) without changing what SUM(TAKE(...))
reads. docs/excel-semantics/references.md "Calling a call's result".

## Acceptance criteria
- [x] probes/chained-calls-builtin-take, -drop, -trimrange and -xlookup pass and leave eval/baseline/probes.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
A called result's callee is now read by `Model::evaluate_callee` (functions/logical/lambda.rs), which
reads it as SUBTOTAL does (`evaluate_picked_reference_arg`): a TAKE, DROP, TRIMRANGE or XLOOKUP over a
range is the range it picks, so `TAKE(A1:A3,1)(2)` and the others are `#REF!`. Both callee sites
(node_frames.rs `LambdaCallKind`, `call_callee_as_reference`) use it; SUM(TAKE(..)) is unchanged.
Eval: probes known failures 2167 -> 2160 (fixed 7: chained-calls-builtin-take B1, -drop B1:B2,
-trimrange B1:B3, -xlookup B1), no regressions.
