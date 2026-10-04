---
id: T-0931
title: "Parser: bound formula nesting depth (stack overflow on deeply nested parentheses)"
milestone: M1
lane: engine
status: todo
priority: P0
size: S
tier: opus
depends_on: []
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
A formula such as `=((((...1...))))` nested a few hundred levels deep must return a parse error, never crash the process.

## Context
Found while testing T-0104 through Python: `wb.set("A1", rs.Formula("=" + "(" * 500 + "1" + ")" * 500))` segfaults (stack overflow in the recursive-descent parser); 200 levels works. Violates CLAUDE.md prime directive 5 (untrusted input: no unbounded recursion) and SPEC-0001's test plan ("deeply nested formulas return `EditError`, never panic").

## Acceptance criteria
- [ ] Parser enforces a maximum nesting depth (pick a limit that Excel's own 64-level function nesting cap sits well under; cite the evidence) and returns a typed parse error beyond it.
- [ ] Also covers nested function calls, arrays `{}` and unary-operator chains; the evaluator's recursion over the resulting AST is bounded by the same limit.
- [ ] Engine unit tests plus a facade test (`Workbook::set` with `Input::Formula` at depth 10_000 returns `EditError`).
- [ ] `make check` green; reviewers approve.

## Out of scope
Matching Excel's exact error message for over-nested formulas.
