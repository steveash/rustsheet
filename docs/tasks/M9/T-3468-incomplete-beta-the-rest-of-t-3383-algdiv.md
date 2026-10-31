---
id: T-3468
title: "Incomplete beta, the rest of T-3383: algdiv's stored intermediates from the r4-algdiv pins, Excel's direct bgrat increment to the bit, lnGamma(a0 < 1) in bpser, the b < 1 bpser ceiling, bfrac and bup arithmetic"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Models, with exact, within-4-ulp and within-15-digit counts on held-out probes, for the parts of Excel's incomplete beta
that T-3383 (r4-algdiv, item 2 for the density) left open, so the engine tasks have targets.

## Context
- Split from T-3383 (size), itself split from T-2512. T-3383 answered item 2's open question for the density only
  (docs/research/t2464/r4-algdiv/RESULTS.md): the density's lnB at min < 1 < 8 < max is pinned uniquely on 156 of 160
  shapes, sits -4 to +3 ulps from the correctly rounded lnB, and its offset is uncorrelated between neighbouring b
  (storage noise, not an approximation error). Every other item is carried over unchanged. Work in
  docs/research/t2464/ (new `r5-*` directories), importing r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py`,
  r3-bgrat's `data.py`/`bscore.py` and r4-algdiv's `scan.py`. Method: docs/research/excel-kernel-playbook.md.
- Item 1 leftovers:
  - **Excel's direct bgrat increment D at b ≠ 1/2, to the bit.** Best form so far is RN(F·Q(b, z_s)/Q(b, z_e)),
    246/1,881/2,357 of 2,908 (probe t2512-r3-bgrat-split). Excel sits a median 4 units of 2^-53 below the exact value.
    Next: x runs of consecutive doubles at a fixed (a, b0) to find the stored intermediate; grat1's continued
    fraction in x87 with Excel's EXP for r.
  - **n > 1 bup terms** (2 <= b < 40, a > 15, x > 0.7) and the F, BINOM and NEGBINOM callers of L140 bup+bgrat: probe
    them and check that RN(W + D) holds, with W the n-term bup on Excel's density in bup's orientation.
  - **Excel's GAMMA.DIST at fractional b < 1 near P = 1**: probe t2512-r3-bgrat-qgamma column A is 1,268 of 4,980
    cells off in the engine, mostly 1-2 ulp, 117 at the 15th digit. Record it in the incomplete-gamma entries, or file
    an engine task if it is not one the existing incomplete-gamma tasks own.
  - **F at a df of 1** reaches bgrat at b = 1/2. T-2509's `bgrat_half` on F's (x, y) misses Excel
    (`FDIST(x,1,9)`: Excel 0.190947117169724, bgrat 0.19094711716972382, 7 ulp), so find what F does differently
    (statistics.md "Student t tails are one kernel call").
- Open items carried over from T-2512 unchanged. Item 2 is now the whole visible gap on L140's one-term cells (r3-bgrat
  finding 2):
  2. **algdiv for b0 >= 8.** Score x87 storage variants of TOMS algdiv (r3-m1-residual's `algsearch.py` grid,
     `algdiv_v.py`) per shape against r4-algdiv's 156 pinned lnB values (`scan_a.json`, `scan_hold.json`; in-sample a
     only, hold-out once at the end), not per row. Then check the winner in bpser's L (the cdf; implied L -1.88, +1.65
     and +6.05 units on r2-bpser-small's runs). Rejected in r4-algdiv: the correctly rounded lnB (34 and 37 of 80
     shapes), fl(GL(a) + CR algdiv) (22, 35), TOMS-d (26, 30), Excel's GAMMALN difference past 8 in either
     association (0-1).
     - At a = 1 or b = 1 the density's lnB is -LN(the other shape) at any size, not algdiv (statistics.md "Beta density
       for min(a,b) <= 1").
  3. **lnΓ(a0 < 1) inside bpser's L** depends on the shape: GL below 0.3, fl(gamln1(v) - LN v) at w2's (0.5, 6.25). On
     the old 1 < b0 < 8 cells GL53 gets 3,365 against r2-bratio's 4,038 of 7,455. Probe bpser at fixed b0 with a0
     stepping by single ulps around 0.3, 0.5 and 0.7.
  4. **The b < 1 bpser ceiling.** At any constant L, b < 1 shapes top out at 55-84% bit-exact. The misses come in
     sawtooth blocks, each fixed by moving s or E one ulp, so the cause is one stored intermediate (r2-bpser-small
     `misspos_*.out`, `fixwhich.out`).
  5. **bfrac's fraction arithmetic.** The residual is white, with sd 3.8 units at (100, 120). Try the original Fortran
     BFRAC compiled for x87, and the per-iteration quantities (w4-fractions section 7).
  6. **bup's arithmetic in bup+bgrat** (T n <= 30: 356 of 1,344 bit-exact) and the a0 < 1e-15 band at
     0.04 <= y < 0.25 (Excel 1e-14 to 3.8e-13 relative off the true value, a problem in bgrat's incomplete-gamma ratio).
- Rules: positive controls before trusting a method; a real hold-out; record negatives (decision 0056); truncate
  formula literals to 15 digits as Excel does.

## Acceptance criteria
- [ ] Each item above has a recorded result in a RESULTS.md: a model with counts on a held-out probe, or the negatives
      tried with their counts.
- [ ] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's T-2464 entries are updated with what is found.
- [ ] Engine follow-ups are filed for any solved piece, or the owning engine task's criteria are amended.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-3382 ports the L140 one-term composition).
