---
id: T-3712
title: "Incomplete beta L140 bup+bgrat with n >= 2 bup terms: Excel's density times R_i EXP(fl(i LN y)) (T-3568)"
milestone: M9
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3382]
---

## Goal
BETA.DIST's cdf, F.DIST/F.DIST.RT, BINOM.DIST's cdf and NEGBINOM.DIST's cdf on bratio's L140 bup+bgrat branch with
n >= 2 bup terms (2 <= b0 < 40, a0 > 15, x0 > 0.7 on the oriented call) are within 15 digits of Excel on most cells.
Today's engine is visibly off on 1,371 of the 1,650 column-A cells of t3568-r5-bup-hold (tune 1,403 of 1,650, fresh
672 of 825; docs/research/t2464/r5-bup `engcols.py` → `engcols.out`, from `make eval` on the T-3568 branch).

## Context
- T-3568 (docs/research/t2464/r5-bup/RESULTS.md; statistics.md "Incomplete beta: the integrated model", bullet "L140
  bup+bgrat with n >= 2 bup terms"): with n = int(b0), c = b0 - n (c = 1 and n - 1 at an integer b0), the cell is
  RN(fl(ret · S) + D):
  - ret = x87(P · fl(x0·y0)/c), with P the beta density at (y0; c, a0) (bup's orientation, as in T-3382);
  - S = 1 + Σ_{i<n} fl(R_i · EXP(fl(i · LN(y0)))) in doubles, R_i the running double product of
    fl(apb + l)/fl(ap1 + l), with TOMS bup's k and eps = 1e-15 break;
  - D the bgrat increment I_x0(a0, c), as T-3382 computes it; at c = 1 it is x0^a0 (Q(1, z) = e^-z), not
    BETA.DIST(x0, a0, 1, TRUE), which is the a0 = 1 complement there and returns 0 below about 5e-17.
- Measured with Excel's own density for P: hold-out 394/1,163/1,383 of 1,650 (exact/4 ulp/15 digits), post-freeze
  sweep 186/574/696 of 825, against 102/575/892 and 56/286/450 for the research kernel (`score_hold.out`). The
  engine's own density is within 15 digits of Excel's P (column C) on 1,621 of 1,650 hold cells (`engcols.out`), so a port on it can
  reach most of these cells. Measure it first, as T-3382's `engdens.py` did for n = 1. The density's lnB at a0 >= 8
  is T-3569.
- T-3382 ports the n = 1 composition: extend its code path (`bratio/bup.rs`, `bratio/mod.rs`
  `Branch::L140BupBgrat`) to n terms, do not copy it (directive 3). Excel's EXP and LN are `x87_exp.rs` and the
  engine's LN model.
- Cases: probes-numeric t3568-r5-bup-tune, -hold and -fresh (column A; columns B and C are Excel's D and P there).

## Acceptance criteria
- [ ] L140 bup+bgrat with n >= 2 takes the composition above for BETA.DIST, F, BINOM and NEGBINOM cdf cells. The
      per-case `make eval` deltas on the three t3568-r5-bup cases' column A are in the Outcome, with more cells within
      15 digits than before on each case.
- [ ] The Outcome reports the port's hold-out score (exact/4 ulp/15 digits) on t3568-r5-bup-hold and its Gate A
      envelope on t3568-r5-bup-fresh (within 15 digits of Excel or within 1e-9 of the 50-digit value; decision 0188
      R4.5), listing every point outside, because the port runs on the engine's density, not Excel's P.
- [ ] No probes-numeric or gated cell goes from within 15 digits to visibly off. Any that does is listed with
      Excel / new / exact values, or the branch is narrowed.
- [ ] Unit tests pin a few Excel cells from the cases above, with the case and cell named.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The n >= 10 leftover that grows with the binade of (n - 1)·|ln y0| (open in T-3568's RESULTS.md, section 5).
- The density's lnB (T-3569) and D to the bit (T-3383).
- Column B's own `BETA.DIST(x, a, 1, TRUE)` cells that are visibly off (47, the a0 = 1 complement): T-3764.
