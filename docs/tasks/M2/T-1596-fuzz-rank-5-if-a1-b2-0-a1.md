---
id: T-1596
title: "Fuzz: RANK(5,@IF(A1:B2>0,A1:ANK)) reaches a scalar-context cell with a 2x2 array (debug_assert panic)"
milestone: M2
lane: functions-stat
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Evaluating `=RANK(5,@IF(A1:B2>0,A1:ANK))` (the fuzz input with an explicit `@`) in a cell never
panics, in debug or release builds, and gives Excel's result.

## Context
Nightly fuzz run 38678152308 (2026-10-15, `main` 6b35f595), target `formula_eval`, artifact
`crash-49854f45b6bbbe8fb31a06d89f1b952570036d6f`. The input is the 26 bytes
`RANK(5,@IF(A1:B2>0,A1:ANK))` (base64 `UkFOSyg1LEBJRihBMTpCMj4wLEExOkFOSyk=`); the target sets it
as `=...` in `C3` of its small model (`fuzz/fuzz_targets/formula_eval.rs`). Reproduced by the
steward on 6e51f2bf with a debug engine unit test (`model._set("C3", ...)`, then `evaluate()`):

    panicked at crates/rustsheet-engine/src/model.rs:926:25:
    Larger-than-1x1 array reached scalar-context cell (sheet=0, row=3, column=3, 2x2);
    implicit intersection was expected to collapse it.

The same assertion as T-1303, which fixed `RANK(5,IF(A1:C1>0,A1:<undefined name>))` by making
`static_analysis_rank` return `Unknown` when `ref` has an unknown static shape. With an explicit
`@` in front of the IF, the call is still taken for a scalar while evaluation returns a 2x2 array:
without the `@`, `=RANK(5,IF(A1:B2>0,A1:ANK))` gives `#VALUE!` in C3 and D3 (no panic). Find
how `@` on an unknown-shape operand is analysed (static analysis of the implicit-intersection
operator, and how RANK's `ref` reads an intersected non-reference array) and make static analysis
and evaluation agree. Excel's value: check probes/rank-unknown-shape-ref and
probes/rank-nonref-*; if they do not cover `@` on the ref, use the excel-probe skill.

## Acceptance criteria
- [ ] The reproducer is a unit test (decision 0035) that passes in a debug build.
- [ ] `RANK`, `RANK.EQ` and `RANK.AVG` with an `@`-prefixed unknown-shape `ref` agree in static
      analysis and evaluation; the value matches excel-graph evidence, or the case is left pending
      and the task blocked.
- [ ] `make check` green; reviewers approve.

## Out of scope
- AGGREGATE (T-0296); the un-prefixed form (T-1303, done).
