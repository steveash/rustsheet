---
id: T-1140
title: "Probe union and intersection precedence inside one parenthesis and chained cuts of a union"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles how Excel groups a union and an intersection written in one parenthesis and how chained intersections cut a union, replacing the "Engine behavior, not probed" bullet in docs/excel-semantics/references.md with evidenced rules.

## Context
M1 scope "Operators and precedence: the range/union/intersection operators". references.md, union entry, bullet "Engine behavior, not probed: the union is the loosest operator inside its parenthesis and the intersection binds tighter, so `(A1,B1 C1)` is `(A1,(B1 C1))`. A union cut by an intersection runs the cut on each pair of areas, drops pairs with no shared cell, and gives `#NULL!` when nothing is left. Further cuts can bring several areas down to one (`(A1:A12,D1:D12) A4:D4 D4` is D4)." The parser reads the union this way since T-0193 (`Parser::parse_parenthesized`). probes/multiarea-intersect (SPEC-0004 row 11) has only `=SUM((A1:A3,B1:B3) A1:B2)`; no probe fixes the grouping or the empty-cut result. Use ISREF, ROWS, SUM and FORMULATEXT, which tell a reference from an error and show Excel's stored text. A form Excel for the web refuses on load is isolated as a one-formula scratch case and recorded, not committed (as in T-0998).

Follow the `excel-probe` skill. New case probes/union-cut-precedence (numbers in A1:D12) with at least:
- `=SUM((A1,B1 C1))`, `=SUM((A1:B1,B1:C1 B1))`, `=SUM(((A1,B1) B1))`, and FORMULATEXT of each.
- `=(A1:A12,D1:D12) A4:D4 D4`, `=SUM((A1:A12,D1:D12) A4:D4)`, `=ROWS((A1:A12,D1:D12) A4:D4 D4)`.
- Empty cuts: `=SUM((A1,B2) C3)`, `=ISREF((A1,B2) C3)`, `=IFERROR(SUM((A1,B2) C3),"e")`, `=ERROR.TYPE(SUM((A1,B2) C3))`.
- A cut leaving one of two areas: `=SUM((A1:A3,C1:C3) A2:B2)`, `=(A1:A3,C1:C3) A2:B2` (a single-area result as a cell value).

## Acceptance criteria
- [ ] probes/union-cut-precedence is committed with an excel-graph oracle sidecar for every form that loads; refused forms are recorded in references.md with message and date.
- [ ] references.md replaces the "Engine behavior, not probed" bullet with the evidenced grouping, empty-cut and chained-cut rules, citing the case cells (VERIFIED (web)), and keeps only the parts still unprobed, named as such.
- [ ] Engine mismatches are recorded with `make eval-baseline` as new-coverage failures only, and a follow-up engine task owning them is filed and named in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Union reaching functions (SPEC-0004 tasks T-1062 to T-1073) and negated union operands (settled by T-0998, T-1031).
- Parser or evaluator changes (file the follow-up instead).
