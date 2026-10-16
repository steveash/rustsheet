---
id: T-1659
title: "Lotus-style dotted ranges: SUM(A1.B2) and ROWS(A1..B2) read A1:B2"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A dotted A1 range (`A1.B2`, `A1..B2`, `B2.A1`) in a formula is the range `A1:B2`, as in Excel, and FORMULATEXT shows it with `:`.

## Context
Found by T-1652 (docs/excel-semantics/references.md "R1C1-shaped sheet and defined names"). Excel for the web: `SUM(A1.B2)` is 7 and FORMULATEXT `=SUM(A1:B2)`; `ROWS(A1..B2)` is 2; `SUM(B2.A1)` is `=SUM(A1:B2)` (probes/lotus-dot-range). A defined name shaped like that is shadowed by the range: with `R1.C1` defined, `=R1.C1*1` is #SPILL! and `=C1.C1*1` is 0 (probes/r1c1-name-r1-dot-c1, -r1-dotdot-c1, -c1-c1-dot). Probe first: sheet-qualified forms (`Data!A1.B2`), `$` cells, whole rows or columns, a name as the second part, and whether the shape after the second cell matters (sheet quoting accepts a trailing `.anything`). Code: the A1 lexer (expressions/lexer).

## Acceptance criteria
- [ ] probes/lotus-dot-range and the three r1c1-name dotted cases pass; the probed forms are recorded in the semantics entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
