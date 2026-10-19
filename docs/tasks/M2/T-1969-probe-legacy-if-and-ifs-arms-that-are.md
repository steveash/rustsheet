---
id: T-1969
title: "Probe legacy IF and IFS arms that are other references (name, OFFSET, INDEX, a : range)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy formula places its `@` like Excel when an IF or IFS arm is a reference other than a bare
range: a defined name for a range, OFFSET or INDEX of one, or a `:` range operation.

## Context
Found by T-1935's review. T-1935 passes an IF arm that is a bare or parenthesized range whole
(`is_range_arm`, `returns_range_arm` in
crates/rustsheet-engine/src/expressions/parser/static_analysis/legacy.rs; docs/excel-semantics/references.md
"Legacy IF passes a range arm whole"). The other reference shapes still get the `@` on the arm, and
IFS still intersects its value arms: `=IFS(TRUE,B1:B6)` shows `=@IFS(TRUE,@B1:B6)`. None of these is
probed. Probe, with the layout of probes/legacy-if-arm-range-argument (rows 2 and 10, FORMULATEXT on
row 12), `SUM(IF(TRUE,Rng))` with a defined name Rng = B1:B6, `SUM(IF(TRUE,OFFSET(B1,0,0,6)))`,
`SUM(IF(TRUE,INDEX(B1:C6,0,1)))`, `SUM(IF(TRUE,B1:INDEX(B1:B6,6)))`, `SUM(IFS(TRUE,B1:B6))`,
`MATCH(3,IFS(TRUE,B1:B6),0)` and `=IFS(TRUE,B1:B6)`.

## Acceptance criteria
- [ ] A probe case with oracle values for the shapes above; the engine matches them, or each
      mismatch is recorded in references.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
