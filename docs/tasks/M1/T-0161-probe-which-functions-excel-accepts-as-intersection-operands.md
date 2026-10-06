---
id: T-0161
title: "Probe which functions Excel accepts as intersection operands"
milestone: M1
lane: research
status: done
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

Also probe the value-operator operands refused by inference in T-0148: `=A1 (B1+C1)`,
`=A1 (-B1)`, `=A1 (B1&"")`, `=A1 (B1=1)`, `=A1 (B1%)`.

## Acceptance criteria
- [x] Each candidate probed; result in docs/excel-semantics/references.md.
- [x] `is_reference_operand` matches the evidence; tests added.
- [x] `make check` green; reviewers approve.

## Out of scope
- `@` and union operands (T-0149).

## Outcome
Probed 23 operands in Excel for the web (excel-graph, 2026-10-06), one scratch workbook each.
Accepted (give 2): INDIRECT, CHOOSE, XLOOKUP, LET (with `_xlpm.` names), SWITCH, IFS, TAKE, DROP.
Workbook refused (`FileCorruptTryRepair`): ABS, ROW, IFERROR, IFNA, CHOOSEROWS, CHOOSECOLS, FILTER,
VSTACK, SORT, UNIQUE, and the value operators `(B1+C1)`, `(-B1)`, `(B1&"")`, `(B1=1)`, `(B1%)`.
`is_reference_operand` is now an allowlist of the evidenced reference-returning functions; parser
tests cover every probed form. probes/intersection-function-operands gains H6:H12 (oracle-filled).
Eval: probes known failures 64 -> 67 (3 new-coverage failures: the engine evaluates XLOOKUP, TAKE and
DROP to arrays, filed as T-0165), 0 regressions. Follow-ups: T-0165, T-0166 (spec2xlsx `_xlpm.`).
