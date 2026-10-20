---
id: T-2131
title: "IRR's returned iterate to 15 significant digits (50 cells right to only 13-14 digits)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
IRR returns a value within half a unit of the 15th significant digit of Excel's on the probe and
workbook cells. Excel returns an un-converged iterate, so this means stopping where Excel stops,
not converging further.

## Context
- `financial/financial_util.rs` `compute_irr` runs:
  1. a secant in v = 1/(1+r) from v0 and v0 ± 1e-3, stopping when |Δv| < 1e-7 and |NPV| < 1e-7;
  2. one retry from 0.1;
  3. a Newton fallback.
- docs/excel-semantics/financial.md, "IRR, RATE and XIRR stop early": Excel's solvers return an
  un-converged iterate. T-1533 fitted part of IRR's update and stop rule (decision 0065).
- Visibly-off cells on the 2026-10-20 run:
  - `probes-numeric/m9-fin-irr-rate`: 32;
  - `probes/solver-iteration-traces`: 14 (per-iterate traces);
  - `ironcalc-calc/FINANCIAL/IRR.xlsx`: 8, e.g. L17 `IRR(A17:F17)` Excel -0.6737366751808405,
    ours -0.6737366752054068;
  - `probes/fin-argument-modes`: 2.
- The 24 NPV cells of IRR.xlsx (`NPV(L2,A2:F2)` and similar) evaluate NPV at IRR's root. They
  are near-zero residues that follow the IRR value and should move with it.

## Acceptance criteria
- [ ] Every IRR cell listed above is within half a unit of the 15th significant digit. The fitted
      rule (update, stop test, fallback path) is recorded in financial.md with its cells.
- [ ] The IRR.xlsx NPV residue cells are re-measured and the Outcome states where they land.
- [ ] Regressions follow decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR (T-2045).
- RATE (T-2005).
- IRR's error and lift edge cases (T-1686, T-2103).

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083). Bit-exact iterates are a follow-up
if the stage-1 fit does not reach them.
