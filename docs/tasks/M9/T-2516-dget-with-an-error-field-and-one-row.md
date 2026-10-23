---
id: T-2516
title: "DGET with an error field and one-row criteria: which error wins (T-1390 follow-up)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A D-function whose `field` is an error and whose criteria (or database) is one row returns the same
error Excel does.

## Context
T-1390 made one-row criteria and one-row databases `#VALUE!` (docs/excel-semantics/statistics.md
"Database criteria"). One owned gen-m9 cell still differs:
gen-m9/7bd219ec0c0c5e22 Calc!A324, whose inner call is
`DGET(Data!A2:A4,GAMMA.DIST(Data!$A$10,42,1e-15,"FALSE"),Data!A7:D7)` (Data!A10 is TRUE, A7:D7 is one
row): Excel `#VALUE!`, ours `#NUM!`. Either our GAMMA.DIST over a boolean cell gives the wrong error, or
Excel checks the criteria shape before the field (`db_query` in functions/database.rs resolves the
field first). Probe both with excel-probe: `GAMMA.DIST(<TRUE cell>,42,1E-15,"FALSE")` alone, and
`DSUM(<db>,NA(),<one-row criteria>)`, `DSUM(<one-row db>,NA(),<criteria>)`, `DSUM(<db>,"Nope",<one-row criteria>)`.

## Acceptance criteria
- [ ] A probe answers the order, recorded in docs/excel-semantics/statistics.md "Database criteria".
- [ ] gen-m9/7bd219ec0c0c5e22 Calc!A324 leaves eval/baseline/gen-m9.txt (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other D-function semantics (T-1390).
