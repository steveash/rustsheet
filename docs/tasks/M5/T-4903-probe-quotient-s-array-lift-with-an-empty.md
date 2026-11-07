---
id: T-4903
title: "Probe QUOTIENT's array lift with an empty argument and SERIESSUM's empty coefficients against an x/n/m error"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Close the two open points T-4873 left: whether a written-empty QUOTIENT argument beside an array lifts to `#N/A`
in each element, and which error wins when SERIESSUM's coefficients are empty and x, n or m is an error.

## Context
- T-4873 review follow-up (PR #3930, excel-fidelity MINOR): probes/t4873-empty-arg-quotient-ceiling Calc!A7:A8
  (`QUOTIENT({7,8},)`, `QUOTIENT(,{2,4})`) spill into occupied cells and show only `#SPILL!`; numbers.md "A
  written-empty argument of a ToolPak function is #N/A" marks the coefficient order UNVERIFIED.
- Ours: `fn_quotient` (mathematical.rs) gives each element `#N/A` through `Model::atp_number`; `series_coefficients`
  (seriessum.rs) refuses empty coefficients, and that error is reported after x, n and m are checked.
- Design MINOR from the same review: `crates/rustsheet-engine/src/functions/logical/let.rs` near line 283 has
  `matches!(node, Node::EmptyArgKind) || self.is_omitted_variable(node)`, a copy of `Model::is_empty_argument`. Call
  the helper instead.

## Acceptance criteria
- [x] A probes case (excel-graph) has `INDEX(QUOTIENT({7,8},),1,2)`, `ROWS(QUOTIENT(,{2;4}))`, and
      `SERIESSUM(1/0,1,1,)`, `SERIESSUM(2,"x",1,)` and `SERIESSUM(2,1,NA(),)`, each in a cell with room to spill.
      Their predictions are committed before the fill, and each cell passes.
- [x] numbers.md drops the UNVERIFIED note or states the order the probe shows.
- [x] let.rs uses `is_empty_argument` at the duplicated test.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 2010+ roundings (T-4873 settled them).

## Outcome
Probe `t4903-empty-arg-quotient-seriessum-lift` (excel-graph, 2026-11-07), predictions committed before the fill, 8/8
held: `INDEX(QUOTIENT({7,8},),1,2)` `#N/A`, `ROWS(QUOTIENT(,{2;4}))` 2, and SERIESSUM with empty coefficients gives
`#DIV/0!`, `#VALUE!`, `#N/A` for an erroring x, n, m (x, n, m rank first, as the engine does; B4:B5 added after fidelity review because B3 `#N/A` could not tell an m error from the empty coefficients, A3 COLUMNS added for the lift). numbers.md lost its
UNVERIFIED note. The let.rs duplicate was already gone on main (it uses `is_empty_argument`). No engine change; no
math sidecar entries (no Gate A one-call cell); `math --check` current.
