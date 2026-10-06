---
id: T-1024
title: "Union operands: accept repeated unary minus, refuse negated operand cut by intersection, keep nested parentheses in printing"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Union operand parsing and printing match Excel for repeated unary minus, a negated operand cut by an intersection, and nested parentheses.

## Context
Found by T-0998. Rule: docs/excel-semantics/references.md "Union: parsing and evaluation", operand forms. Failing: probes/union-operand-double-minus H1 (Excel `#VALUE!`, engine `#ERROR!`), probes/union-operand-nested-parens H2 and H6 (FORMULATEXT keeps every parenthesis level). Excel refuses (FileCorruptTryRepair) `(-A1,B1) A1`; the engine accepts it, so make the parser refuse it (no probe case possible). Code: `is_union_operand`, `group` in expressions/parser/reference_operators.rs.

## Acceptance criteria
- [ ] The three failing cells pass and the baseline shrinks.
- [ ] `(-A1,B1) A1` is a parse error; a unit test covers it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
