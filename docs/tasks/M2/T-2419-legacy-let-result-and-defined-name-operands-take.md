---
id: T-2419
title: "Legacy @: LET result and defined-name operands take the automatic @, LAMBDA call removes all @"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy cell's automatic `@` matches Excel for a LET returning a range, a defined name as an operand, and a LAMBDA call in the formula.

## Context
T-2387 probes (probes/legacy-at-let-lambda-dynamic-funcs P, S, BI; probes/legacy-at-defined-name-dynamic-funcs A, D, G, J, M, P, V) and docs/excel-semantics/references.md "Dynamic-array-only functions and LET, LAMBDA and names" list the cells still failing; code in `static_analysis/legacy.rs`. Names are M8 scope, so land the LET and LAMBDA part first.

## Acceptance criteria
- [x] A LET returning a range takes the `@` itself (probes/legacy-at-let-lambda-dynamic-funcs P, S); a LAMBDA call removes every `@` (BI). The defined-name operand moved to T-2440.
- [ ] (moved to T-2440) The other dynamic-array-only functions (SORTBY, XMATCH, VSTACK, DROP, TEXTSPLIT, ...) are probed and added to `NO_AUTOMATIC_AT` where they remove the `@`.
- [x] The LET/LAMBDA probe cells pass (`make eval-show`), or the unmatchable ones are recorded as deviations.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
LET returning a range takes the `@` itself (`let_returns_range`) and a LAMBDA call removes every automatic `@` (`removes_automatic_at`), on import and in the export mirror. probes/legacy-at-let-lambda-dynamic-funcs now 163/163 (eight cells fixed). Defined names and the remaining function probes moved to T-2440.
