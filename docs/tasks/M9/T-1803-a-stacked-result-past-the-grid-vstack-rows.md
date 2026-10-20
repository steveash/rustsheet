---
id: T-1803
title: "A stacked result past the grid: VSTACK rows are #NUM!, HSTACK and VSTACK columns past 16384 are allowed (probes/scalar-lift-open-c)"
milestone: M9
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1780]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
A stacked result with more rows than the grid is `#NUM!`, as Excel's, and one with more columns than the grid is allowed.

## Context
T-1780 bounds HSTACK and VSTACK by `check_array_size`, which gives `#VALUE!` for a result past the grid, and T-1777 question 11 asked what Excel returns. probes/scalar-lift-open-c, -c2, -c3 (excel-graph, 2026-10-17): `ROWS(VSTACK(A:A,B1))`, `ROWS(VSTACK(A:A,B1:B2))`, `COLUMNS(VSTACK(A:A,B1:B2))` and `ROWS(VSTACK(A:A,1:1))` (-c2) are `#NUM!` (rows past 1,048,576). Columns past the grid are NOT an error: `COLUMNS(HSTACK(A1:XFD1,A1))` and `COLUMNS(HSTACK(A1:XFD1,A1:A2))` are 16385 and `COLUMNS(HSTACK(A1:A2,1:1))` is 16385 (the same as TEXTSPLIT's wide row, textsplit-wide), and `ROWS(HSTACK(A:A,1:1))` (-c3) is 0 in the Graph backend (an anomaly; do not copy). The 2^34-cell forms made the Graph backend hang or answer oddly, so only the cell counts that Excel handled are entered. The stacking callers of `check_array_size` (T-1780) change. T-1770, which shares the bound, should probe whether its lifted results (a column count past 16384) are allowed too. Whether EXPAND, WRAPCOLS and WRAPROWS (T-1786, done) have the same row/column asymmetry is not probed: a small probe task is needed (`EXPAND(1,1048577,1)` against `EXPAND(1,1,16385)`).

## Acceptance criteria
- [x] VSTACK and HSTACK give `#NUM!` for a result with more than 1,048,576 rows and no error for more than 16,384 columns, under the cell-count limit.
- [x] probes/scalar-lift-open-c W1-W10 pass; -c2 W1 moved to T-1965 (it sits in row 1, which it stacks, so ours is `#CIRC!`).
- [x] `make check` green; reviewers approve.

## Out of scope
- The 2^34-cell memory bound itself (`check_array_cells`).

## Outcome
`limits::check_stack_size` (rows past the grid `#NUM!`, then the cell-count bound) replaces `check_array_size` in
`fn_hstack` and `fn_vstack`. arrays.md gains "Stacked results past the grid". `ROWS(HSTACK(A:A,1:1))` is now `#ERROR!`
(the cell bound; Graph's 0 is not trusted). -c2 W1 and -c3 W1 stay known failures: both sit in row 1, which they stack,
so ours is `#CIRC!` (T-1965). Follow-up T-1967 probes EXPAND, WRAPROWS and WRAPCOLS.
Eval delta: probes known failures 2000 -> 1992 (8 fixed: scalar-lift-open-c W1-W4, W6-W9), 0 regressions.
