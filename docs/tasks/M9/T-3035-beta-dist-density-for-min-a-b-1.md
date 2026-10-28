---
id: T-3035
title: "BETA.DIST density for min(a,b) <= 1 and BINOM.DIST at k = 0 or n: the visibly-off residue after T-2503 and T-2645 (algdiv for max > 8, shape 1 against a large shape)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
The 193 visibly-off cells below, BETA.DIST(x,a,b,FALSE) with min(a,b) <= 1 and BINOM.DIST(0,n,p,FALSE), agree with
Excel to 15 significant digits, or the Outcome records, per group, the candidate forms tried and their counts, and
files the group that stays off as a Phase 1b task with that evidence.

## Context
- Filed by T-2981 (eval of 2026-10-28 at `main` 9586958b). T-2503 shipped model M2 for min(a,b) <= 1
  (`functions/statistical/beta_density.rs` `beta_density_small_shape`) and T-2645 Excel's log1p; T-2821, the
  bit-level residue, is `defer: phase-1b`, so these visibly-off cells had no open owner. They are in three groups:
  1. **algdiv, min < 1 < 8 < max (133 cells):** t2513-r3-m1-residual-a 86, -hold 28, t2513-verify-r3-m1-residual-hold
     15, m9-beta-dist-a0p0001 2 (`BETA.DIST(0.1,0.0001,50,FALSE)` Excel 5.727663409649543e-6, ours
     5.727663409649533e-6) and m9-beta-dist-a50 2. M2's lnB there is fl(GL(min) + algdiv(min, max)) with TOMS algdiv
     in doubles, "the frozen choice" (docs/research/t2464/r3-m1-residual/RESULTS.md section 1: algdiv OPEN). Its
     "algdiv accurate" column is a first candidate. T-2512 (deferred) owns cracking Excel's algdiv; this task owns
     only the visible cells, so a form that wins on them without being Excel's is enough.
  2. **Shape 1 against a large shape (29 cells):** t2464-r2-log1p-grid 13, -hold 7, -runs 7
     (`BETA.DIST(p,1,2^k+1,FALSE)`, k = 10 to 12, results near 1E-190) and t2513-r3-m1-residual-a 1
     (`BETA.DIST(0.337...,1,333,FALSE)`). M2 takes lnB = -LN(max) at min = 1; the density is EXP(fl(b-1)·log1p(-x)
     - lnB), so one ulp of log1p(-x) moves the result by about (b-1) ulp.
  3. **BINOM.DIST(0,n,p,FALSE), n = 2^k (27 cells):** the same 27 p values in the same three log1p probes. Excel's
     value is EXP(fl(n·l)) with l its log1p(-p) (statistics.md "BINOM.DIST pmf: closed forms at k = 0 and k = n");
     docs/research/t2464/r2-log1p/RESULTS.md section 4 lists exactly 27 p where the log1p model's series tail s is
     1 ulp off Excel's (16 exact ties of r + s, 11 just past a midpoint). Groups 2 and 3 are one cause: those 27
     log1p misses times n. Fixing the tie rule of the log1p tail (r2-log1p section 6 hypotheses) fixes both.
  - The rest: t2513-r3-m1-residual-a 1 and t2464-w2f-absweep 4 (max <= 8, the GAMMALN form; shapes next to
    (0.5, 6.25), T-2821's "newly" list) and m9-beta-dist-a50's 2 swapped cells.
- Code: `beta_density.rs` (`beta_density_small_shape`, `algdiv`), the Excel log1p primitive from T-2645
  (`x87` modules), `binom.rs` for BINOM.DIST(0,n,p).

### Cases owned (T-2981)
Visibly-off cells per case at the routing eval (only these functions' cells; other cells of the same case are
routed to the task named in T-2981's Outcome):
- probes-numeric/m9-beta-dist-a0p0001: BETA.DIST density, min(a,b) <= 1 2
- probes-numeric/m9-beta-dist-a50: BETA.DIST density, min(a,b) <= 1 2
- probes-numeric/t2464-r2-log1p-grid: BETA.DIST density, min(a,b) <= 1 13, BINOM.DIST pmf at k = 0 or n 13
- probes-numeric/t2464-r2-log1p-hold: BETA.DIST density, min(a,b) <= 1 7, BINOM.DIST pmf at k = 0 or n 7
- probes-numeric/t2464-r2-log1p-runs: BETA.DIST density, min(a,b) <= 1 7, BINOM.DIST pmf at k = 0 or n 7
- probes-numeric/t2464-w2f-absweep: BETA.DIST density, min(a,b) <= 1 4
- probes-numeric/t2513-r3-m1-residual-a: BETA.DIST density, min(a,b) <= 1 88
- probes-numeric/t2513-r3-m1-residual-hold: BETA.DIST density, min(a,b) <= 1 28
- probes-numeric/t2513-verify-r3-m1-residual-hold: BETA.DIST density, min(a,b) <= 1 15

## Acceptance criteria
- [ ] Each group above is either fixed (its cells within 15 digits in `make eval`, tests pin rows from the probes)
      or has its candidates scored in the Outcome and a `defer: phase-1b` follow-up filed with `make new-task`.
- [ ] No cell of these cases or of a gated suite that is within 15 digits today goes visibly off (`make eval-check`).
- [ ] statistics.md's M2 and log1p entries record what is found.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit-exactness (T-2821, T-2512, deferred).
- min(a,b) > 1 densities (T-2507).

## Visible-parity stage
Target (decision 0114): the 193 cells above within 15 digits, or each remaining group explained with its evidence
and handed to Phase 1b.
