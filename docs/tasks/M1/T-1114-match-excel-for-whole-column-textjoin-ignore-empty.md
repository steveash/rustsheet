---
id: T-1114
title: "Match Excel for whole-column TEXTJOIN(ignore_empty FALSE), XIRR and XNPV"
milestone: M1
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXTJOIN with ignore_empty FALSE, XIRR and XNPV give Excel's result for a whole-column or
whole-row argument, whatever the sheet's used area is.

## Context
Found by the Excel-fidelity reviewer on T-0049. These walkers clip a whole-row/column range to the
used area (`Model::clip_to_used_area`, `Worksheet::used_area_end`) but, unlike SUM or COUNT, act on
empty cells: TEXTJOIN(",",FALSE,D:D) adds a delimiter per empty walked cell
(functions/text/concatenate.rs), XIRR reads empty as 0 and XNPV turns empty into #NUM!
(functions/financial/mod.rs, `get_array_of_numbers_xirr` / `_xpnv`). Their result therefore
depends on where the clip ends (a cell-less row from an xlsx style-only `<row>` moves it), which
Excel's cannot. Excel over a whole column: TEXTJOIN would exceed 32,767 characters (expected
#VALUE!? unverified); XIRR/XNPV unverified. Probe first (`excel-probe` skill).

## Acceptance criteria
- [ ] Oracle-backed probe cases for `TEXTJOIN(",",FALSE,D:D)`, `XIRR(A:A,B:B)` and `XNPV(0.1,A:A,B:B)` on a small sheet; quirks recorded in `docs/excel-semantics/`.
- [ ] The engine matches them, independent of rows that exist without cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
