---
id: T-0998
title: "Probe union operand edge cases: double minus, percent, nested parentheses, negated operand cut by intersection"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior for union operand forms T-0193 chose without evidence is known and the engine matches it.

## Context
T-0193 accepts one unary minus on a union operand and refuses `(--A1,B1)`, `(A1%,B1)`; it keeps one level of written parentheses (`((A1,B2))` prints as `(A1,B2)`); behavior of `(-A1,B1) A1` is unknown. See docs/excel-semantics/references.md, "Union: parsing and evaluation" (engine choices). Code: `is_union_operand`, `group` in expressions/parser/reference_operators.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [x] A probe suite case per form above, with oracle values; references.md updated to VERIFIED or corrected.
- [x] Any mismatch is filed as an engine task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes union-operand-{double-minus,nested-parens,neg-cut-group} oracle-filled; Excel refuses `(A1%,B1)` and `(-A1,B1) A1`/`B1` as unloadable workbooks (documented, no case possible). references.md updated. Engine mismatches: 3 cells (new-coverage), filed as T-1024.
