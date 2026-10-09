---
id: T-1114
title: "Match Excel for whole-column TEXTJOIN(ignore_empty FALSE), XIRR and XNPV"
milestone: M1
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXTJOIN with ignore_empty FALSE and XNPV give Excel's result for a whole-column or whole-row
argument, whatever the sheet's used area is; XIRR's behavior is probed and recorded (its fix is
T-1124).

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
- [x] Oracle-backed probe cases for `TEXTJOIN(",",FALSE,D:D)`, `XIRR(A:A,B:B)` and `XNPV(0.1,A:A,B:B)` on a small sheet; quirks recorded in `docs/excel-semantics/`.
- [x] The engine matches TEXTJOIN and XNPV, independent of rows that exist without cells (XIRR: T-1124).
- [x] `make check` green; reviewers approve.

## Out of scope
The XIRR engine fix (T-1124: needs a probe of mismatched empty pairs first).

## Outcome
Probe probes/whole-column-empty-walkers and probes/xirr-xnpv-whole-column-nogaps (excel-graph): `TEXTJOIN(",",FALSE,D:D)` is #CALC!, whole-row
TEXTJOIN joins all 16,384 cells, XNPV is #NUM! on empty cells and a whole column counts every cell
(`XNPV(0.1,A:A,B:B)` is #NUM! on a sheet holding only A1:B2), XIRR skips empty cells and accepts
dates before the first. TEXTJOIN now adds the delimiters of the empty cells its used-area clip skips (bounded by
the 32,767 limit), so it no longer depends on where the clip ends; XNPV's range reader applies its empty-cell #NUM! to the
cells the clip skips. XIRR's empty-cell skip needs a
probe of mismatched pairs first and is filed as T-1124 (its cells would still miss on XIRR's
convergence). Recorded in text.md and financial.md.
Eval delta: +2 probe cases (24 cells: 15 pass, 9 XIRR new-coverage failures), 0 regressions.
