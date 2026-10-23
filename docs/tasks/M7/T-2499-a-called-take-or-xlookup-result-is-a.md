---
id: T-2499
title: "A called TAKE or XLOOKUP result is a reference: TAKE(A1:A3,1)(2) and XLOOKUP(1,A1:A3,A1:A3)(2) are #REF!"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A called TAKE or XLOOKUP result is the reference it returns, so the call is `#REF!` as in Excel:
`=TAKE(A1:A3,1)(2)` and `=XLOOKUP(1,A1:A3,A1:A3)(2)`.

## Context
T-2405 chained calls after the built-ins `is_reference_operand` accepts. Those two cells are new
failures in eval/baseline/probes.txt (probes/chained-calls-builtin-take, -xlookup, excel-graph
2026-10-23): ours `#NAME?`, because `LambdaCallKind`'s callee is read by
`evaluate_reference_arg` (functions/util.rs). There an unlifted TAKE gives its value
(`reference_producer`) and XLOOKUP is not a reference case at all, so the call sees an array or a
value and calls it like a LET-bound value. Read them as references in the callee
(`call_callee_as_reference` / node_frames.rs `LambdaCallKind`) without changing what SUM(TAKE(...))
reads. docs/excel-semantics/references.md "Calling a call's result".

## Acceptance criteria
- [ ] probes/chained-calls-builtin-take and -xlookup pass and leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
