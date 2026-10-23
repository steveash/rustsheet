---
id: T-2507
title: "Beta density for min(a,b) > 1: Loader's saddle point with Excel's bd0 (T-2464 E5)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2501]
---

## Goal
BETA.DIST(x,a,b,FALSE) with both shapes > 1 follows Excel's Loader (2000) saddle-point structure,
(a+b-1)·dbinom_raw(a-1, a+b-2, x, fl(1-x)), with Excel's bd0. The same function is the brcomp prefactor of bfrac
(T-2508).

## Context
- T-2464 research, step E5 of docs/research/t2464/r2-impact/RESULTS.md section 4.
- The rule is docs/excel-semantics/statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point (T-2464)":
  - q = fl(1 - x), and M = fl(n·x) and fl(n·q), with n = fl(a+b) - 2;
  - bd0's log form when |k - M| >= 0.2·(k + M): t = fl(k·LN(fl(k/M))), then t + (M - k) unrounded;
  - Loader's series below that cut;
  - the constant LN(fl(a+b) - 1) + stirlerr terms - lf/2, with R <= 2.7.1's lf from Excel's LN, and exact stirlerr.
- Model: `docs/research/t2464/r2-density-large/model.py` (LDe: `stir=exact, lf=LF0`). It scores 5,765/25,254/31,385 of
  32,264 cells, against CR 3,573/26,024/29,280.
- The status is PARTIAL:
  - The per-shape constant is unexplained, from -8 to +8 units, so LDe loses 770 within-4-ulp cells to CR.
  - Its hold-out probe does not test bd0 (verifier).
  - On pre-T2464 cells LDe is no better than CR: it fixes 149 and breaks 17, against 148 and 4 (r2-impact
    `pdf_large.out`).
  - The engine's powf × tgamma is closer to Excel's last bits on w2f's max < 8 runs: 1,222 bit-exact, against LDe's
    719.
- So this task is a visible-parity step, not a bit-exactness one. Ship the structure if it wins on visible cells with
  no gated loss. Otherwise record the comparison and leave the density to T-2513.
- r2-impact's expected change: -148 visibly-off pre-T2464 cells (it breaks 25 there) and -5,007 in the w2f probes.
- Code: `incomplete_beta::beta_density` (and statrs above a + b = 171.6). Exact stirlerr can come from Excel-independent
  code (a table plus a Stirling series); it must be accurate to well under an ulp of the result.

## Acceptance criteria
- [ ] On the 32,264 a, b > 1 density cells of r2-density-large's data, the engine is within 15 digits on at least
      31,385 and bit-exact on at least 5,765.
- [ ] The Outcome reports, per function and suite, the visible change on pre-T2464 cells against today's engine.
- [ ] No gated known-failure line is added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pinning the per-shape constant or the series branch (T-2513).
- The T density (T-2513).

## Visible-parity stage
Decision 0083: this task is done when every BETA.DIST(.,FALSE) cell with min(a,b) > 1 that it owns is within half a unit
of the 15th significant digit of Excel's value, or the cells still off are named in the Outcome and handed to T-2513.
r2-impact counts 261 visibly off pre-T2464 (224 at max >= 8, 37 at max < 8). Bit-level criteria move to a follow-up
task without the campaign key.
