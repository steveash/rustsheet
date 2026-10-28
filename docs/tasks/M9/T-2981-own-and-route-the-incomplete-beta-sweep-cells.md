---
id: T-2981
title: "Own and route the incomplete-beta sweep cells no kernel task names (70 probes-numeric cases, about 88,500 visibly-off cells)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
Every visibly-off cell of the incomplete-beta research sweeps that no kernel task names has one owner that will fix
it: each sweep case below is assigned to the ported-kernel task whose branch computes it, and that task's text names
the case. Cells no planned branch covers get a new `campaign:` task. After this, the BETA.DIST, F.DIST, BINOM.DIST
and NEGBINOM.DIST gaps are fully owned by tasks the queue picks before the milestone order.

## Context
- Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a; calipers, poi and corpus-bench not fetched, which
  hold none of these cells): 113 in-scope functions are below the Phase 1 rate. BETA.DIST (52,755 visibly-off of
  207,001 attributed cells, 74.5% within 15 digits), F.DIST (38,102 of 72,627, 47.5%), BINOM.DIST (3,456) and
  NEGBINOM.DIST (1,648) hold about 94% of the visibly-off cells of all 113.
- 88,502 of those cells are in the 70 probes-numeric sweep cases below, and no open, non-deferred task
  names them (T-2821 named some whole cases in its Context; it is now `defer: phase-1b`). Under the exit-owner rule
  (scripts/exit_owners.py `owned`), a task line that names a case stem and no cell owns the whole case, so this task
  owns all of these cases until each is handed on. The per-function totals: BETA.DIST 46,122, BINOM.DIST 2,919, F.DIST 37,771, F.DIST.RT 167, NEGBINOM.DIST 1,523.
- The fix is the ported incomplete-beta chain (T-2464's E1-E8): T-2506 and T-2507 are in flight, then T-2508,
  T-2509, T-2510 and T-2511 (all P1 after the triage), with T-2847 moving F, BINOM and NEGBINOM onto the bratio
  branches, T-2790 for the F density and T-2585 for the t density. The routing below is a first guess from each
  sweep's name and its research note; verify it before handing a case on.
- The single visibly-off gen-m9 cells of these functions (about 25: F.DIST, F.DIST.RT, FDIST, BINOMDIST,
  NEGBINOMDIST) are left to the T-0087 sweep.

Cases (visibly-off cells per function at the triage eval):
- **First guess: T-2847 (F onto the bratio branches), then T-2508 (bfrac) and T-2510 (bgrat)** (33,879 cells in 23 cases):
  - probes-numeric/t2586-r4-f-brcomp-big: F.DIST 1,549
  - probes-numeric/t2586-r4-f-brcomp-hold: F.DIST 742
  - probes-numeric/t2586-r4-f-power-a: F.DIST 4,360
  - probes-numeric/t2586-r4-f-power-hold: F.DIST 1,273
  - probes-numeric/t2586-r4-f-small-df-a: F.DIST 1,872
  - probes-numeric/t2586-r4-f-small-df-hold: F.DIST 543
  - probes-numeric/t2586-r5-f-bigconst-a: F.DIST 3,838
  - probes-numeric/t2586-r5-f-bigconst-hold: F.DIST 1,419
  - probes-numeric/t2586-r5-f-combine-a: F.DIST 3,464
  - probes-numeric/t2586-r5-f-combine-hold: F.DIST 968
  - probes-numeric/t2586-r5-f-edges-a: F.DIST 806
  - probes-numeric/t2586-r5-f-edges-hold: F.DIST 565
  - probes-numeric/t2586-r5-f-final-hold: F.DIST 1,220
  - probes-numeric/t2586-r5-f-smallr-a: F.DIST 5,600
  - probes-numeric/t2586-r5-f-smallr-hold: F.DIST 1,151
  - probes-numeric/t2586-verify-r4-f-brcomp-hold: F.DIST 523
  - probes-numeric/t2586-verify-r4-f-power-hold: F.DIST 805
  - probes-numeric/t2586-verify-r4-f-small-df-hold: F.DIST 104
  - probes-numeric/t2586-verify-r5-f-bigconst-hold: F.DIST 686
  - probes-numeric/t2586-verify-r5-f-combine-hold: F.DIST 701
  - probes-numeric/t2586-verify-r5-f-edges-hold: F.DIST 443
  - probes-numeric/t2586-verify-r5-f-final-hold: F.DIST 567
  - probes-numeric/t2586-verify-r5-f-smallr-hold: F.DIST 680
- **First guess: T-2507 (Loader saddle point with Excel bd0, min(a,b) > 1; in progress)** (27,211 cells in 8 cases):
  - probes-numeric/t2513-r3-loader-bd0-a: BETA.DIST 7,749
  - probes-numeric/t2513-r3-loader-bd0-b: BETA.DIST 113
  - probes-numeric/t2513-r3-loader-bd0-hold: BETA.DIST 4,633
  - probes-numeric/t2513-r3-loader-const-diverse1: BETA.DIST 5,630
  - probes-numeric/t2513-r3-loader-const-grid1: BETA.DIST 1,275
  - probes-numeric/t2513-r3-loader-const-sweep1: BETA.DIST 5,918
  - probes-numeric/t2513-verify-r3-loader-bd0-hold: BETA.DIST 950
  - probes-numeric/t2513-verify-r3-loader-const-hold: BETA.DIST 943
- **First guess: T-2507 (the binomial pmf is the same bd0 saddle point), then T-2847** (8,041 cells in 4 cases):
  - probes-numeric/t2513-r3-binom-pmf-a: BETA.DIST 1,945, BINOM.DIST 1,213
  - probes-numeric/t2513-r3-binom-pmf-hold: BETA.DIST 1,608, BINOM.DIST 628, NEGBINOM.DIST 780
  - probes-numeric/t2513-r3-binom-pmf-n: BINOM.DIST 997, NEGBINOM.DIST 629
  - probes-numeric/t2513-verify-r3-binom-pmf-h: BETA.DIST 73, BINOM.DIST 54, NEGBINOM.DIST 114
- **First guess: T-2503 residue at min(a,b) <= 1 (split the visible part of T-2821) or T-2507 for min(a,b) > 1** (5,668 cells in 13 cases):
  - probes-numeric/m9-beta-dist-a0p0001: BETA.DIST 2
  - probes-numeric/m9-beta-dist-a10: BETA.DIST 9
  - probes-numeric/m9-beta-dist-a100: BETA.DIST 25
  - probes-numeric/m9-beta-dist-a2: BETA.DIST 4
  - probes-numeric/m9-beta-dist-a2p5: BETA.DIST 6
  - probes-numeric/m9-beta-dist-a5: BETA.DIST 8
  - probes-numeric/m9-beta-dist-a50: BETA.DIST 19
  - probes-numeric/t2464-w2f-absweep: BETA.DIST 2,547
  - probes-numeric/t2464-w2f-unit-shape: BETA.DIST 7
  - probes-numeric/t2464-w2f-xruns-2: BETA.DIST 2,910
  - probes-numeric/t2513-r3-m1-residual-a: BETA.DIST 88
  - probes-numeric/t2513-r3-m1-residual-hold: BETA.DIST 28
  - probes-numeric/t2513-verify-r3-m1-residual-hold: BETA.DIST 15
- **First guess: T-2506 / T-2847 (bpser and bup branches with Excel log1p)** (5,318 cells in 4 cases):
  - probes-numeric/t2464-r2-log1p-grid: BETA.DIST 13, BINOM.DIST 13
  - probes-numeric/t2464-r2-log1p-hix: BETA.DIST 5,264
  - probes-numeric/t2464-r2-log1p-hold: BETA.DIST 7, BINOM.DIST 7
  - probes-numeric/t2464-r2-log1p-runs: BETA.DIST 7, BINOM.DIST 7
- **First guess: T-2790 (F density), after T-2507** (3,743 cells in 5 cases):
  - probes-numeric/t2513-r3-fdensity-a: BETA.DIST 978, F.DIST 884
  - probes-numeric/t2513-r3-fdensity-hold: F.DIST 522
  - probes-numeric/t2513-r3-fdensity-holdsw: F.DIST 59
  - probes-numeric/t2513-r3-fdensity-sw: F.DIST 865
  - probes-numeric/t2513-verify-r3-fdensity-hold: F.DIST 435
- **First guess: T-2507 (density, min(a,b) > 1)** (2,889 cells in 1 cases):
  - probes-numeric/t2464-r2-density-large-p1: BETA.DIST 2,889
- **First guess: T-2585 (t density) / T-2790 (F density)** (679 cells in 3 cases):
  - probes-numeric/t2513-r3-tdensity-a1: F.DIST 115
  - probes-numeric/t2513-r3-tdensity-a2: F.DIST 240
  - probes-numeric/t2513-r3-tdensity-hold: F.DIST 324
- **First guess: T-2219 (F.DIST to 15 digits) / T-2790** (610 cells in 5 cases):
  - probes-numeric/t2220-f-dist-sweep-cdf: F.DIST 110
  - probes-numeric/t2220-f-dist-sweep-d1-1: F.DIST 47, F.DIST.RT 14
  - probes-numeric/t2220-f-dist-sweep-d2-1: F.DIST 76, F.DIST.RT 11
  - probes-numeric/t2220-f-dist-sweep-pdf: F.DIST 212
  - probes-numeric/t2220-f-dist-sweep-rt: F.DIST.RT 140
- **First guess: T-2847 (bpser branches)** (446 cells in 2 cases):
  - probes-numeric/t2464-r2-bpser-small-tiny: BETA.DIST 223
  - probes-numeric/t2464-r2-bpser-small-tiny2: BETA.DIST 223
- **First guess: T-2510 (bgrat) / T-2506** (18 cells in 2 cases):
  - probes-numeric/t1538-beta-dist-at-excel-inv: BETA.DIST 4, F.DIST 3, F.DIST.RT 2
  - probes-numeric/t2464-r2-bratio-x29: BETA.DIST 9

## Acceptance criteria
- [ ] For each case above, `make eval-show CASE=probes-numeric/<stem>` (or the sweep's research note) shows which
      kernel branch computes its visibly-off cells, and the owning task's text names the case stem, with the
      visibly-off count, in a "Cases owned (T-2981)" list. The case is then removed from this task's list.
- [ ] Cells that no open task's planned branch covers (for example, the min(a,b) <= 1 cells T-2503's M2 leaves
      visibly off, now that T-2821 is parked) get a new task with `campaign: accuracy` or `visible` and the
      `## Visible-parity stage` section, filed with `make new-task`.
- [ ] Afterwards `python3 scripts/exit_owners.py --all-off` (T-0086), or the T-0090 method until it lands, shows no
      unowned visibly-off cell in these cases.
- [ ] No engine code changes; `make check` green; reviewers approve.

## Out of scope
- Fixing the kernels (the chain tasks above).
- Bit-level residue of these sweeps (Phase 1b tasks).

## Visible-parity stage
Target (decision 0114): every visibly-off cell named here has an owning campaign task the queue picks before the
milestone order. This task changes no value; its stage only makes sure it is picked first in M9.

## Cells added by the T-0087 sweep (2026-10-28)
Visibly-off Excel cells no open task owned on the eval of 2026-10-28 at `main` c2b6143a (`python3 scripts/exit_owners.py --all-off`; calipers, poi and corpus-bench not fetched): F.DIST.RT 1, GAMMALN 1. This task now owns them:
- probes-numeric/m9-f-dist-rt: Calc!A19
- probes-numeric/t2464-r2-bpser-small-gl: Calc!A3188
Routing for these two (first guess, verify as for the cases above): the F.DIST.RT cell goes with the F upper tail on the ported kernel (T-2510 bgrat / T-2977); the GAMMALN cell is a GAMMALN kernel value inside the bpser sweep, for a GAMMALN follow-up of T-2123 (finished) to file.
