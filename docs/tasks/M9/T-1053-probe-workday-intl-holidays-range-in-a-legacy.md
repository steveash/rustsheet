---
id: T-1053
title: "Probe WORKDAY.INTL holidays range in a legacy formula (xlsx-authored, expect whole range)"
milestone: M9
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior is recorded for a legacy `=WORKDAY.INTL(C2,6,1,D2:D5)` (holidays range, expected whole range, 42438 for the poi/59106 layout).

## Context
Follow-up of T-0983 (review). WORKDAY.INTL's `holidays` is a whole-range argument by analogy with NETWORKDAYS.INTL and the Excel-cached WORKDAY case (poi/59106.xlsx Test!B2); no Excel evidence covers WORKDAY.INTL itself. Typed formulas never get implicit intersection, so the probe needs an xlsx-authored legacy formula (or an Excel-cached workbook).

## Acceptance criteria
- [ ] Evidence for WORKDAY.INTL is recorded in docs/excel-semantics/dates.md ("WORKDAY holidays take the whole range"), which names WORKDAY.INTL.
- [ ] The engine matches, or the gap is baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
