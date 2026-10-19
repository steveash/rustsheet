---
id: T-1967
title: "Probe whether EXPAND, WRAPROWS and WRAPCOLS past the grid share VSTACK's rows-#NUM!/columns-allowed rule"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel oracle values say whether EXPAND, WRAPROWS and WRAPCOLS past the grid follow VSTACK's rule (rows past it `#NUM!`, columns past it allowed), and the engine matches them.

## Context
From T-1803. arrays.md "Stacked results past the grid" (probes/scalar-lift-open-c, -c2): VSTACK rows past the grid are `#NUM!`, HSTACK columns past it are allowed; T-1803 put that rule in `limits::check_stack_size` for HSTACK and VSTACK only. EXPAND, WRAPROWS and WRAPCOLS (T-1786) still use `check_array_size`, which is `#VALUE!` past the grid for rows and columns alike. Probe with the excel-probe skill: `ROWS(EXPAND(1,1048577,1))`, `COLUMNS(EXPAND(1,1,16385))`, `COLUMNS(EXPAND(A1:A2,2,16385))`, `ROWS(WRAPCOLS(SEQUENCE(16385),1))` (16385 columns), `COLUMNS(WRAPROWS(SEQUENCE(16385),16385))`, `EXPAND(1,1E9,1E9)`, and VSTACK of an input wider than the grid (`COLUMNS(VSTACK(TEXTSPLIT(REPT(",",16384),","),1))`).

## Acceptance criteria
- [x] A probe case with excel-graph values covers the formulas above; arrays.md records the answer.
- [x] If they follow VSTACK's rule, the three functions call `check_stack_size`; every probe cell passes or has an owning task.
- [x] `make check` green; reviewers approve.

## Out of scope
- The cell-count bound itself (`check_array_cells`).

## Outcome
Probe `probes/expand-wrap-past-grid` (excel-graph, 2026-10-19): EXPAND rows past the grid are `#NUM!`, columns past it
are allowed; WRAPROWS/WRAPCOLS columns past the grid are allowed (their rows cannot pass it); a VSTACK of an input
wider than the grid is allowed. So all three follow VSTACK's rule: EXPAND, WRAPROWS and WRAPCOLS now call
`check_stack_size`. Recorded in arrays.md "Stacked results past the grid". Eval: all 10 probe cells pass (new case, no
baseline change).
