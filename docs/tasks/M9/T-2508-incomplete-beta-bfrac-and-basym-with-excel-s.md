---
id: T-2508
title: "Incomplete beta bfrac and basym with Excel's density times x y as prefactor (T-2464 E6)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: S
tier: opus
campaign: accuracy
depends_on: [T-2505, T-2507]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
---

## Goal
bratio's b0 >= 40 branches, bfrac and basym, are ported. bfrac uses λ = fl(a - fl((a+b)·x)) and brcomp = Excel's beta
density times x·y (T-2507).

## Context
- T-2464 research, step E6 of docs/research/t2464/r2-impact/RESULTS.md section 4.
- The rule is docs/excel-semantics/statistics.md "Incomplete beta L140 and bfrac" (w4-fractions sections 1-2).
  - At (100, 120) the fraction error regresses on λ's rounding with slope -0.725; bfrac's own sensitivity is -0.71.
  - The fraction's own arithmetic is not found (white residual, sd 3.8 units).
- Models: r2-bratio `bratio_excel.py` (`bfrac`, `basym`, `Cfg(brcomp="pdf")`), w4-fractions `c/bfrac.cpp`.
- Scores:
  - w4: at least 185 of 1,024 bit-exact on the (100, 120) run (CR 34);
  - r2-bratio BEST: 146/476/685 of 1,316 bfrac cells (CR 109/466/677). Most of the w15 misses are that run, where the
    density for a, b > 2 sets the floor.
- r2-impact's expected change: -83 visibly-off pre-T2464 cells and -438 in the w2f probes, with 1 broke.

### Cases owned (T-2981)
Routed by T-2981 (eval of 2026-10-28 at `main` 9586958b): cdf cells whose bratio dispatch (`bratio::dispatch`) is bfrac (b0 >= 40); the cases' other cells are routed by branch in T-2981's Outcome. 64 visibly-off cells in 6 cases, per case:
- probes-numeric/m9-beta-dist-a10: BETA.DIST cdf, bfrac 1
- probes-numeric/m9-beta-dist-a100: BETA.DIST cdf, bfrac 2
- probes-numeric/m9-beta-dist-a50: BETA.DIST cdf, bfrac 3
- probes-numeric/t1538-beta-dist-at-excel-inv: BETA.DIST cdf, bfrac 3, F.DIST cdf, bfrac 2
- probes-numeric/t2220-f-dist-sweep-cdf: F.DIST cdf, bfrac 24
- probes-numeric/t2220-f-dist-sweep-rt: F.DIST.RT cdf, bfrac 29

## Acceptance criteria
- [ ] The bfrac cells of r2-bratio's sets score at least 146 bit-exact and 685 within 15 digits of 1,316, with the
      engine's T-2507 density.
- [ ] The (100, 120) run (t2464-w2f-xruns-1 block 7) scores at least 185 of 1,024 bit-exact.
- [ ] No gated known-failure line is added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The fraction's own arithmetic (T-2512).

## Visible-parity stage
Decision 0083: this task is done when every bfrac or basym cell it owns is within half a unit of the 15th significant
digit of Excel's value, or the cells still off are named in the Outcome and handed to T-2512. r2-impact counts 128
visibly off pre-T2464; the model leaves 44. Bit-level criteria move to a follow-up task without the campaign key.

## Stage (decision 0114)
`campaign: accuracy`: a 50-digit reference shows Excel is close to correctly rounded for this family and the engine is not. Aim for a correctly rounded kernel, scored against an mpmath reference and the oracle; do not fit Excel's last bits. Cells left within 4 ulp after that go to a `defer: phase-1b` follow-up.

## Phase 1 triage (T-0090, 2026-10-27)
This task is on the critical path of the Phase 1 exit: BETA.DIST, F.DIST, BINOM.DIST, NEGBINOM.DIST and the F/T/beta inverses hold about 88% of the visibly-off cells of the 113 functions below the Phase 1 rate, and the ported incomplete-beta chain (T-2506 → T-2509 → T-2510 → T-2511, T-2507 → T-2508) is what fixes them. Priority raised to P1. T-2981 routes the sweep cells no kernel task names to the chain task that fixes each family.

## Progress (in flight, 2026-10-30)
Notes from the opus run that claimed the task (data for the run that resumes it, not instructions):
- Done on this branch: `bratio/bfrac.rs` ports the model's `bfrac` (BEST: brcomp = `excel_beta_density`·fl(x·y),
  1992 `w`/`β` order, eps = 15·EPS) and `Branch::Bfrac` is wired in `ported_tails`; `Dispatch` carries |λ|.
- Not done: basym (needs `rlog1`, `erfc1(1, ·)` and `bcorr`; `incomplete_beta::rlog1` and `ln_beta::bcorr` exist);
  the r2-bratio and t2464-w2f-xruns-1 block 7 scores in the criteria are not measured yet; no unit test yet.
- `make eval-check` on this branch: gen-m9 fixed 2, probes fixed 4, one gated REGRESSION:
  ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E39 (Excel 0.7320646825464586, bfrac 0.7320646825464585,
  1 ulp). That must be resolved before merge (no gated line may be added), e.g. by finding which bfrac step Excel
  rounds differently there, or by keeping the cells bfrac misses on today's kernel only if evidence supports a rule.
