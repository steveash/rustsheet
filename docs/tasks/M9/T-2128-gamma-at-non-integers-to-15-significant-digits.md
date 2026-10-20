---
id: T-2128
title: "GAMMA at non-integers to 15 significant digits on an accurate kernel, off statrs gamma"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
GAMMA at non-integer arguments is within half a unit of the 15th significant digit of Excel's
value. It no longer comes from statrs' Lanczos `gamma`.

## Context
- `functions/statistical/gamma.rs` `fn_gamma` calls `statrs::function::gamma::gamma`.
- On the 2026-10-20 run, 240 GAMMA cells are visibly off. Excel is close to correctly rounded:
  against a 50-digit reference, Excel's median relative error is 1.5e-16 and the engine's is
  4e-14.
- Non-integer cells (this task):
  - `probes-numeric/t1042-gamma-half-int`: 57 cells, e.g. `GAMMA(-1.5)` Excel 2.363271801207354,
    ours 2.363271801207348;
  - `t1042-gamma-unit-interval`: 2;
  - `t1042-gamma-negatives-edges`: 9;
  - `ironcalc-calc/STATISTICAL/GAMMA.xlsx` B9, B10 and B16;
  - `BETA_GAMMA.xlsx` rows with non-integer x.
- Integer cells are T-0973's (Excel computes them as the exact downward product).
- `libm` (`tgamma`) is already a dependency and used in `incomplete_beta.rs`. Reuse it, or an
  existing engine kernel, rather than adding one (directive 3).
- Notes: docs/excel-semantics/statistics.md, "GAMMA and GAMMALN sweep (T-1042)".

## Acceptance criteria
- [ ] Every non-integer GAMMA cell in the cases above is within half a unit of the 15th
      significant digit of Excel's value. Show it with `make eval-show`, or with T-0061's bucket
      if that has landed.
- [ ] No GAMMA cell, and no cell of a function built on this kernel, becomes visibly off. Any
      bit-exact cell that moves stays within 4 ulp and follows decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GAMMA at integers (T-0973).
- GAMMALN (T-1529).
- Last-ulp fitting of the non-integer cells: file it as a follow-up if it is not already exact.

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083).
