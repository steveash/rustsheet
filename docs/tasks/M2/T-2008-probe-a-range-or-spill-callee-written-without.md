---
id: T-2008
title: "Probe a range or spill callee written without parentheses: Sheet1!:(2), :A2(2), (2)#"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's value and FORMULATEXT for a range, or a cell with a spill operator, written as a callee without parentheses
are probed, recorded in docs/excel-semantics/references.md and matched.

## Context
T-1806's Excel-fidelity review (MINOR): `=Sheet1!$A$1:$A$2(2)`, `=$A$1:A2(2)` and `=$A$1(2)#` are `#ERROR!` (parse
errors) in ours, before and after T-1806. Excel makes `(A1:A3)(2)` `#REF!` (references.md "Calling a parenthesized
operand", T-0149) and `$D$1(1)` `#REF!` (probes/cell-call-edges-a, T-1806), so the unparenthesized range call may be
`#REF!` too, or Excel may refuse the workbook (the case then has no oracle; record that, as T-1806 did for
`R1C1(1)`). Also probe the lower-case row shapes `=a0(1)` and `=a1048577(1)`, to check FORMULATEXT's casing. Use the
`excel-probe` skill, one small case per risky shape. Code: `reference_or_call_operand` (parser/operands.rs).

## Acceptance criteria
- [ ] Each shape has an oracle-backed probe cell or a recorded refusal, and the evidenced cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shapes T-1806 probed.
