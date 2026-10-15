---
id: T-1631
title: "SHEET of a reference returned by INDEX or IF, and of a union"
milestone: M5
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SHEET resolves the sheet of a reference returned by INDEX or IF and of a union (its first area).

## Context
Found by T-1627: probes/volatile-and-info-rand-true-false-sheet D18 (`SHEET(INDEX(T!A1:B2,1,1))` 2), D19 (`SHEET(IF(TRUE,U!A1))` 3) and D26 (`SHEET((T!A1,T!B2))` 2); ours gives `#N/A`, `#N/A` and `#VALUE!` (docs/excel-semantics/information.md, "SHEET of a reference").

## Acceptance criteria
- [ ] The three failing cells pass; baseline updated with `make eval-baseline`
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other SHEET forms probed by T-1627 (they match Excel) and SHEETS.
