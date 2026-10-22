---
id: T-2294
title: "GCD, LCM and MULTINOMIAL are #VALUE! when every argument is a lone blank cell (MULTINOMIAL(Z200), GCD(Z200))"
milestone: M6
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GCD, LCM and MULTINOMIAL give Excel's `#VALUE!` when no argument is anything but a lone blank cell, and keep reading a
blank as 0 everywhere else.

## Context
Found by T-2275's probe (`probes/t2275-address-bounds` column J, excel-graph, 2026-10-21; Z200 and Z201 blank, Z210 = 0).
docs/excel-semantics/coercion.md "GCD / LCM / MULTINOMIAL range cells" has the rule (added by T-2275):
- `#VALUE!`: `MULTINOMIAL(Z200)` (J1), `MULTINOMIAL(Z200:Z200)` (J15), `MULTINOMIAL(Z200,Z201)` (J20), `GCD(Z200)` (J21),
  `LCM(Z200)` (J23), `MULTINOMIAL(IF(TRUE,Z200))` (J25), `GCD(Z200:Z200)` (J28).
- A blank is 0 beside anything else, or inside a multi-cell range: `MULTINOMIAL(Z200,2)`, `(2,Z200)`, `(Z200:Z201)` and
  `(Z200:Z201,2)` are 1, `GCD(Z200,4)` and `LCM(Z200,4)` 4, `MULTINOMIAL(Z210)` (a 0 cell) 1.
Ours gives 1 for the MULTINOMIAL cells and 0 for `GCD(Z200:Z200)`; `GCD(Z200)` and `LCM(Z200)` are already `#VALUE!` (MULTINOMIAL in crates/rustsheet-engine/src/functions/math_and_trigonometry/multinomial.rs
pushes 0 for every blank). Also owns gen-m6/be0e3a1b0fc3f7a0 Calc!A167
`=ADDRESS(9,3,4,(Data!A5=MULTINOMIAL(Data!$D12)),T("1E3"))`: Excel `#VALUE!` because `MULTINOMIAL(<blank>)` is; ADDRESS itself is
right (`ADDRESS(9,3,4,FALSE,"1E3")` = `'1E3'!R[9]C[3]`, J2).
Unprobed: how the lone-blank `#VALUE!` ranks against a later negative or omitted argument; keep the existing error order for
anything with a non-blank argument.
GCD and LCM share code in the same directory; find the common helper before adding a flag to each (CLAUDE.md directive 3).

## Acceptance criteria
- [ ] The probes/t2275-address-bounds cells J1, J15, J20, J25 (MULTINOMIAL) and J28 (`GCD(Z200:Z200)`) pass; J21 and J23
      (`GCD(Z200)`, `LCM(Z200)`) already do, and gen-m6/be0e3a1b0fc3f7a0 Calc!A167 leaves
      eval/baseline/gen-m6.txt; no new known failure in any suite.
- [ ] coercion.md's entry names where the rule is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ADDRESS (T-2275).
