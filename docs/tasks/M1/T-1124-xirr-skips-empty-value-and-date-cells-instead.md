---
id: T-1124
title: "XIRR skips empty value and date cells and accepts dates before the first"
milestone: M1
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
XIRR over ranges with empty cells, including whole columns and rows, gives Excel's result instead of #NUM!.

## Context
probes/whole-column-empty-walkers (excel-graph, 2026-10-07, T-1114): `XIRR(A1:A3,B1:B3)` with A3 and B3 both empty equals `XIRR(A1:A2,B1:B2)` (0.09999999403953552), and `XIRR(A:A,B:B)` / `XIRR(12:12,13:13)` give finite results, so Excel skips the empty cells; the engine reads them as 0 (`get_array_of_numbers_xirr`, functions/financial/mod.rs: `|| Ok(Some(0.0))`) and gives #NUM!. Unprobed and needed first: a pair where only the value or only the date is empty, an empty first cell, and text cells. XIRR also accepts a date before the first date (probes/xirr-xnpv-whole-column-nogaps A4 = -0.01750176399946212; `fn_xirr` returns #NUM! for it), which is why whole-column F3 and E7 (row 12 has date 1100) still fail after empties are skipped. XNPV matches since T-1114. The values will still miss in the last digits until XIRR's early stop is matched (financial.md, "IRR, RATE and XIRR stop early"; E4 shows it on the non-empty pair).

## Acceptance criteria
- [ ] Probe of mismatched-empty pairs filled from excel-graph and recorded in financial.md.
- [ ] XIRR skips empty cells and accepts early dates as Excel does: whole-column-empty-walkers F3, F6, F9, E7 and xirr-xnpv-whole-column-nogaps A4, A8 are no longer #NUM!.
- [ ] `make check` green; reviewers approve.

## Out of scope
XIRR's convergence (last digits).
