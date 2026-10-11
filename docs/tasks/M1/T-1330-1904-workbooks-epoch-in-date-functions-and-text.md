---
id: T-1330
title: "1904 workbooks: epoch in date functions and TEXT date tokens, negatives as minus (SPEC-0005 step 4)"
milestone: M1
lane: functions-date
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-1105]
spec: docs/specs/SPEC-0005-workbook-date-system.md
---

## Goal
In a 1904 workbook, date functions and TEXT date tokens use the 1904 epoch, and negative serials format as a minus plus the absolute value, as probed.

## Context
SPEC-0005 rollout step 4 (docs/specs/SPEC-0005-workbook-date-system.md); evidence probes/date-system-1904-negatives (excel-graph): the failing cells A1, A2, A6, A7, A14 (TEXT negatives), A8, A9 (TEXT epoch), A10-A12 (DATE, YEAR, DATEVALUE); A4, A5, A13 and A15 already pass (the engine prints a minus in either system), and A18 is T-1329's. docs/excel-semantics/dates.md "The 1904 date system". The 1462-day shift applies to untrusted serials: checked arithmetic, an Excel error out of range (SPEC-0005). The formatter part builds on T-1105's `FormatContext`; date functions read `WorkbookSettings::date_system`. Split if it is bigger than M.

## Acceptance criteria
- [ ] The listed probes/date-system-1904-negatives cells pass and leave the baseline; probes/date-system-1900-negatives does not regress.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
