---
id: T-2512
title: "Crack the open incomplete-beta pieces: general-b bgrat, algdiv for b0 >= 8, bfrac noise, the b < 1 bpser ceiling (T-2464 follow-up)"
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
that T-2464 left open. The engine tasks T-2505, T-2506, T-2508 and T-2510 then have targets for the cells they cannot
reach.

## Context
- docs/excel-semantics/statistics.md "Incomplete beta: the integrated model, what is open and what is rejected
  (T-2464)" lists the open items with measurements. Work in docs/research/t2464/ (a new `r3-*` directory), with
  r2-bratio's `bratio_excel.py` as the base model (import, do not copy). Method: docs/research/excel-kernel-playbook.md.
- Open items, largest first:
  1. **bgrat for b ≠ 1/2.** Excel's L140 bup+bgrat scores 212/2,208/2,402 of 3,303 and its bgrat 989/1,761/2,156 of
     2,451, against CR 618/2,814/3,025 and 899/2,072/2,230. There are also 600 x29-probe BETA.DIST/F bgrat cells with
     b ≠ 1/2.
     - First test: r2-tdist's b = 1/2 identity, RN(F·Q_xl/Q), with Q(b, z) taken from Excel's incomplete gamma
       (statistics.md T-1778) in place of ERFC.
     - The (30.7, 1.7, 0.81) runs sit at about -6 ulp, sd 2.9, once literals are truncated (verify-r2-bratio/misparse.out).
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
- Rules: positive controls before trusting a method; a real hold-out (a probe not used for any choice, which round 2
  missed twice); record negatives (decision 0056); truncate formula literals to 15 digits as Excel does.

## Acceptance criteria
- [ ] Each item above has a recorded result in a RESULTS.md: a model with counts on a held-out probe, or the
      negatives tried with their counts.
- [ ] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's T-2464 entries are updated with what is found.
- [ ] Engine follow-ups are filed for any solved piece, or the owning engine task's criteria are amended.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
- The densities (T-2513).

## Stage (decision 0114)
Was `defer: phase-1b` (its cells within 15 digits). T-2510 handed it the bgrat-family cells it could not fix (decision
0083), and those are visibly off, so per the old Deferred note the key is gone and the task takes `campaign: accuracy`:
find Excel's general-b bgrat (and F's b = 1/2 form below) close enough to bring them to visible parity; file the engine
follow-up for what is found.

### Cases owned (from T-2510, eval of 2026-10-31 at `main` 5638599)
cdf cells on bratio's bgrat branches (bgrat, bup+bgrat, L140 bup+bgrat, L140 bup+bup+bgrat) that T-2510 left visibly off
(T-2981's routing, less what T-2510 fixed at a0 = 1/2):
- probes-numeric/m9-beta-dist-a10: BETA.DIST cdf, L140BupBgrat
- probes-numeric/m9-beta-dist-a100: BETA.DIST cdf, Bgrat, L140BupBgrat
- probes-numeric/m9-beta-dist-a5: BETA.DIST cdf, L140BupBgrat
- probes-numeric/m9-beta-dist-a50: BETA.DIST cdf, Bgrat, L140BupBgrat
- probes-numeric/t1538-beta-dist-at-excel-inv: BETA.DIST, F.DIST and F.DIST.RT cdf, L140BupBgrat
- probes-numeric/t2220-f-dist-sweep-cdf: F.DIST cdf, Bgrat, BupBgrat, L140BupBgrat, L140BupBupBgrat
- probes-numeric/t2220-f-dist-sweep-d1-1: F.DIST and F.DIST.RT cdf, Bgrat (a df of 1: b = 1/2)
- probes-numeric/t2220-f-dist-sweep-d2-1: F.DIST and F.DIST.RT cdf, Bgrat, BupBgrat (a df of 1: b = 1/2)
- probes-numeric/t2220-f-dist-sweep-rt: F.DIST.RT cdf, Bgrat, BupBgrat, L140BupBgrat, L140BupBupBgrat
- probes-numeric/t2464-r2-bratio-x29: BETA.DIST cdf, BupBgrat
- probes-numeric/t2464-w2f-absweep: BETA.DIST cdf, L140BupBgrat
- probes-numeric/t2464-w2f-unit-shape: BETA.DIST cdf, BupBgrat
- probes-numeric/t2464-w2f-xruns-2: BETA.DIST cdf, L140BupBgrat

Also open from T-2510: F at a df of 1 reaches bgrat at b = 1/2, but T-2509's `bgrat_half` on F's (x, y) misses Excel
(`FDIST(x,1,9)` Excel 0.190947117169724, bgrat 0.19094711716972382, 7 ulp; F.DIST/F.DIST.RT visibly off 10/11 -> 15/15 on
probes-numeric with it), so F keeps today's kernel there; find what Excel's F does differently (statistics.md "Student t
tails are one kernel call").
