---
id: T-1965
title: "A self-including VSTACK past the grid is #NUM!, not #CIRC! (probes/scalar-lift-open-c2 W1)"
milestone: M9
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`=ROWS(VSTACK(A:A,1:1))` in W1 (inside the row it stacks) is `#NUM!`, as in Excel, not `#CIRC!`.

## Context
From T-1803. probes/scalar-lift-open-c2 W1 (excel-graph, 2026-10-17) is `#NUM!`: Excel answers the stack's size error although W1 is inside `1:1`; ours reports the circular reference first (`#CIRC!`). -c3 W1 `ROWS(HSTACK(A:A,1:1))` is `#CIRC!` too (Excel's Graph 0 is not trusted; arrays.md "Stacked results past the grid"). Find what Excel does with a self-including whole-row or whole-column argument whose result is an error before any cell is read: probe (excel-probe skill) e.g. `=ROWS(VSTACK(A:A,1:1))` and `=ROWS(1:1)`, `=COLUMNS(1:1)`, `=ISREF(1:1)` placed in row 1, against the same formulas outside it, before changing the cycle detection (recalc.md has the existing circular-reference rules).

## Acceptance criteria
- [ ] probes/scalar-lift-open-c2 W1 passes, or a probe shows why it cannot and arrays.md/recalc.md records the rule.
- [ ] No other eval cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 2^34-cell HSTACK bound (DEVIATION).
