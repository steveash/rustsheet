---
id: T-1965
title: "A self-including VSTACK past the grid is #NUM!, not #CIRC! (probes/scalar-lift-open-c2 W1)"
milestone: M9
lane: engine
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
`=ROWS(VSTACK(A:A,1:1))` in W1 (inside the row it stacks) is `#NUM!`, as in Excel, not `#CIRC!`.

## Context
From T-1803. probes/scalar-lift-open-c2 W1 (excel-graph, 2026-10-17) is `#NUM!`: Excel answers the stack's size error although W1 is inside `1:1`; ours reports the circular reference first (`#CIRC!`). -c3 W1 `ROWS(HSTACK(A:A,1:1))` is `#CIRC!` too (Excel's Graph 0 is not trusted; arrays.md "Stacked results past the grid"). Find what Excel does with a self-including whole-row or whole-column argument whose result is an error before any cell is read: probe (excel-probe skill) e.g. `=ROWS(VSTACK(A:A,1:1))` and `=ROWS(1:1)`, `=COLUMNS(1:1)`, `=ISREF(1:1)` placed in row 1, against the same formulas outside it, before changing the cycle detection (recalc.md has the existing circular-reference rules).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 2 cells with a wrong error, blamed on ROWS 2, VSTACK 1, HSTACK 1: -c2 W1 `ROWS(VSTACK(A:A,1:1))` (Excel `#NUM!`, ours `#CIRC!`) and -c3 W1 `ROWS(HSTACK(A:A,1:1))` (Graph 0, ours `#CIRC!`). Only -c2 W1 is this task's visible-parity target: -c3 W1's 0 is a 2^34-cell result arrays.md "Stacked results past the grid" does not trust, and ours is bounded by the cell-count limit (directive 5), so it is not owned here.

## Acceptance criteria
- [x] Visible parity: probes/scalar-lift-open-c2 W1 is `#NUM!` and its line leaves eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] probes/scalar-lift-open-c2 W1 passes, or a probe shows why it cannot and arrays.md/recalc.md records the rule.
- [x] No other eval cell regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 2^34-cell HSTACK bound (DEVIATION).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#CIRC!` where Excel gives a size result), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. -c3 W1 (`ROWS(HSTACK(A:A,1:1))`, Graph 0) is not owned: arrays.md "Stacked results past the grid" does not trust that 0, and the 2^34-cell result is past the directive-5 cell-count bound.

## Outcome
Probe probes/vstack-self-past-grid (excel-graph, 2026-10-21, 11 cells): Excel evaluates every VSTACK
argument, a reference without reading its cells, and checks the stacked rows before reading any, so a
stack past the grid is `#NUM!` even when a reference holds the formula's own cell (W2, W8, W12), an
error argument wins wherever it stands (W3, W4, W10, W11), and an expression argument that reads the
cell is a cycle, 0 (W9). ROWS, COLUMNS and ISREF of the formula's own row read nothing (W5-W7).
`fn_vstack` now evaluates the arguments first (a one-cell reference deferred, a range only to its
corners), checks the references' rows with `check_stack_size`, then reads them; `range_dims` from
drop_take.rs is shared. arrays.md "Stacked results past the grid" records the rule.
Eval delta: probes/scalar-lift-open-c2 W1 fixed (`#NUM!`); the new case passes 10 of 11; W9 is a
new-coverage known failure owned by the circular-reference semantics (ours is `#CIRC!`, Excel 0;
recalc.md "A circular reference without iteration is 0", M3 / T-0401). No regressions.
