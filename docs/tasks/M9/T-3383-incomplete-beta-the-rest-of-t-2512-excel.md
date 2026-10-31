---
id: T-3383
title: "Incomplete beta, the rest of T-2512: Excel's direct bgrat increment to the bit, algdiv for b0 >= 8, lnGamma(a0 < 1) in bpser, the b < 1 bpser ceiling, bfrac and bup arithmetic"
milestone: M9
lane: research
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Models, with exact, within-4-ulp and within-15-digit counts on held-out probes, for the parts of Excel's incomplete beta
that T-2512 round 1 (r3-bgrat, item 1 only) left open, so the engine tasks have targets.

## Context
- Split from T-2512 (size): its item 1 is recorded in docs/research/t2464/r3-bgrat/RESULTS.md and statistics.md's
  "Incomplete beta: the integrated model" entry. Work in docs/research/t2464/ (new `r4-*` directories), importing
  r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py` and r3-bgrat's `data.py`/`bscore.py`. Method:
  docs/research/excel-kernel-playbook.md.
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
  2. **algdiv for b0 >= 8**, in bpser's L and in the density's lnB. It is about 1 unit off TOMS's: implied L is -1.88,
     +1.65 and +6.05 units on r2-bpser-small's runs, and w1 measured ±2 units per b. The positive-argument alnrel
     inside it is not r2-log1p's series. Probe it through a = 1, b > 8 densities with a/b sweeping (0, 0.375].
     - T-2513 round 3 (r3-m1-residual, verifier-confirmed) pinned the density's lnB for min < 1 < 8 < max: under model
       M2 every shape has a unique lnB fitting all its rows, -8 to +6 ulps from TOMS algdiv in doubles (52 shapes;
       `r3-m1-residual/lnbfit.py` rebuilds the git-ignored table `lnbfit_tab.pkl`). The verifier's hold-out adds 10
       shapes, 66 of 270 rows bit-exact, each shape fitting with a shift of -8 to +1 ulp
       (`verify-r3-m1-residual/vshape_vhold.out`). Excel's
       algdiv is within about ±1.7 ulp(algdiv) of the exact ln(Γ(b)/Γ(a+b)). 3,072 x87 storage variants, accurate,
       extended and series-alnrel forms are rejected (`lnbcands.out`, `algsearch.out`). Next: b sweeps at fixed a (one
       a, 40 b, 40 x each) to tell an approximation from a storage pattern.
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
