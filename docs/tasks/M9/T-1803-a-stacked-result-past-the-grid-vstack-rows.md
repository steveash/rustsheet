---
id: T-1803
title: "A stacked result past the grid: VSTACK rows are #NUM!, HSTACK and VSTACK columns past 16384 are allowed (probes/scalar-lift-open-c)"
milestone: M9
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A stacked result with more rows than the grid is `#NUM!`, as Excel's, and one with more columns than the grid is allowed.

## Context
T-1780 bounds HSTACK and VSTACK by `check_array_size`, which gives `#VALUE!` for a result past the grid, and T-1777 question 11 asked what Excel returns. probes/scalar-lift-open-c, -c2, -c3 (excel-graph, 2026-10-17): `ROWS(VSTACK(A:A,B1))`, `ROWS(VSTACK(A:A,B1:B2))`, `COLUMNS(VSTACK(A:A,B1:B2))` and `ROWS(VSTACK(A:A,1:1))` (-c2) are `#NUM!` (rows past 1,048,576). Columns past the grid are NOT an error: `COLUMNS(HSTACK(A1:XFD1,A1))` and `COLUMNS(HSTACK(A1:XFD1,A1:A2))` are 16385 and `COLUMNS(HSTACK(A1:A2,1:1))` is 16385 (the same as TEXTSPLIT's wide row, textsplit-wide), and `ROWS(HSTACK(A:A,1:1))` (-c3) is 0 in the Graph backend (an anomaly; do not copy). The 2^34-cell forms made the Graph backend hang or answer oddly, so only the cell counts that Excel handled are entered. Both `check_array_size` callers that stack (T-1780) and the lifting driver T-1770 (which shares it) must account for the row/column asymmetry, and T-1786 (EXPAND, WRAPCOLS, WRAPROWS) should probe whether its bound is asymmetric too.

## Acceptance criteria
- [ ] VSTACK and HSTACK give `#NUM!` for a result with more than 1,048,576 rows and no error for more than 16,384 columns, under the cell-count limit.
- [ ] probes/scalar-lift-open-c W1-W10 and -c2 pass.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 2^34-cell memory bound itself (`check_array_cells`).
