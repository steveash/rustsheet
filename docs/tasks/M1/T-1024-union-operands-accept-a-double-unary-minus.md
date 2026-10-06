---
id: T-1024
title: "Union operands: accept a double unary minus, refuse negated operand cut by intersection, keep nested parentheses in printing"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Union operand parsing and printing match Excel for a double unary minus, a negated operand cut by an intersection, and nested parentheses.

## Context
Found by T-0998. Rule: docs/excel-semantics/references.md "Union: parsing and evaluation", operand forms. Failing: probes/union-operand-double-minus H1 (Excel `#VALUE!`, engine `#ERROR!`), probes/union-operand-nested-parens H2 and H6 (FORMULATEXT keeps every parenthesis level). Excel refuses (FileCorruptTryRepair) `(-A1,B1) A1` and `(-A1,B1) B1`; the engine accepts it, so make the parser refuse it (no probe case possible). Code: `is_union_operand`, `group` in expressions/parser/reference_operators.rs.

## Acceptance criteria
- [ ] The three failing cells pass and the baseline shrinks.
- [ ] `(-A1,B1) A1` and `(-A1,B1) B1` are parse errors; a unit test covers both.
- [ ] `(--A1,B1)` evaluates to `#VALUE!` (the engine already prints it). Three or more minuses are unprobed: leave as is.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Parser now accepts `(--A1,B1)` (`#VALUE!`, three minuses stay refused), refuses a union with a negated operand as
an intersection operand (`(-A1,B1) A1`, `(-A1,B1) B1`; unit test), and keeps every level of parentheses around a
union for FORMULATEXT (`wrapped` field on `OpUnionKind`). Eval: probes/union-operand-double-minus H1 and
union-operand-nested-parens H2, H6 fixed (3 cells), 0 regressions.
