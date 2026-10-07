---
id: T-0231
title: "Probe and match scalar-error broadcast for comparison and & operators"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A scalar error operand of a comparison operator (`= <> < > <= >=`) or of `&` against an array operand
gives whatever Excel gives (expected: an array of that error, the array operand's size, as for
arithmetic), not a scalar error.

## Context
T-0209 found and implemented this rule for `+ - * / ^` only (`handle_arithmetic` in
`crates/rustsheet-engine/src/arithmetic.rs`; docs/excel-semantics/arrays.md, "A scalar error operand
broadcasts against an array operand"; probes/error-operand-broadcast). The comparison and `&` paths
(`handle_comparison`, `handle_concatenate`) still return the scalar error. Unprobed: write a probe first
(excel-probe skill), e.g. `=SUM(IFERROR(NA()={1;2;3},1))`, `=SUM(IFERROR({1;2;3}<NA(),1))`,
`=SUM(IFERROR(NA()&{"a";"b"},1))`, `=INDEX({1;#DIV/0!}=NA(),2)` (per-element precedence).

## Acceptance criteria
- [ ] Oracle probe committed; docs/excel-semantics/arrays.md entry extended with the result.
- [ ] Comparison and `&` match the probe; unit tests; no new eval F lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Arithmetic operators (done in T-0209).

## Outcome
Probe `probes/error-operand-broadcast-cmp` (oracle: excel-graph) shows comparison and `&` broadcast a scalar error like arithmetic, leftmost error winning per element. The engine already did this (T-0129 ported it for all three paths), so no code change: 15/15 probe cells pass, recorded in `docs/excel-semantics/arrays.md`. Eval delta: probes +15 cells passing, 0 regressions.
