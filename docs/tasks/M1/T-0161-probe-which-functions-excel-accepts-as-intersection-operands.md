---
id: T-0161
title: "Probe which functions Excel accepts as intersection operands"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel-evidenced list of which function calls are valid intersection operands (`=A1:A3 f(...)`), replacing the engine's "refuse only SUM and NA" rule.

## Context
T-0148 refuses value operands but, lacking evidence, accepts every function call except SUM and NA
(`is_reference_operand` in expressions/parser/reference_operators.rs). Evidence exists only for INDEX,
OFFSET, IF (accepted) and SUM, NA (workbook refused). Candidates: INDIRECT, CHOOSE, XLOOKUP, LET,
SWITCH, IFS, TAKE, DROP, CHOOSEROWS, value functions such as ABS, ROW. Refused workbooks give no
value, so record load success/refusal per function (probes/intersection-function-operands).

## Acceptance criteria
- [ ] Each candidate probed; result in docs/excel-semantics/references.md.
- [ ] `is_reference_operand` matches the evidence; tests added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `@` and union operands (T-0149).
