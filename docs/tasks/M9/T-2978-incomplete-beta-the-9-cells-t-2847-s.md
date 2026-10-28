---
id: T-2978
title: "Incomplete beta: the 9 cells T-2847's caller move left 1 to 5 ulp off Excel (L140 at 2.5, 2.5; F bpser; B(a0, 1))"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-2847]
defer: phase-1b
---

## Goal
The cells T-2847's move of F, BINOM and NEGBINOM onto the bratio port left 1 to 5 ulp off Excel are bit-exact again,
by finding the form Excel computes, not by fitting.

## Context
T-2847 (decision 0083's allowance; H221 and H226 under decision 0141) recorded these as known failures. All are within
15 significant digits.
- probes/stat-argument-modes H215 `F.DIST(1.5,5,5,TRUE)` Excel 0.6663904371708331, ours 0.6663904371708333 (+2 ulp);
  H221 `F.DIST.RT(1.5,5,5)` and H226 `FDIST(1.5,5,5)` Excel 0.33360956282916693, ours 0.33360956282916665 (-5 ulp).
  L140 bup(0.5, 2.5, 0.6, 0.4, 2) + bpser(2.5, 0.5, 0.4) at x = 0.6000000000000001; the 40-digit value there is
  0.33360956282916662, so Excel is the inaccurate one. docs/research/t2464/r2-impact/RESULTS.md section 6
  hypothesis 1: F's x as 1/(1 + (d2/d1)/X), or bup's W rounding at b0' = 0.5, n = 2.
- gen-m9 0a614cfe974b59c3 A19 `F.DIST.RT(10,3,4)` (+1 ulp), 05d44f6dd6cfc567 A175 `FDIST(1E+300,Data!B8,1)` (-1),
  27601819397dd56e A49 FDIST (-1), 4ab2ba98d26c8a5e A157 `FDIST(LET(v,PI(),Data!B$12),3,8)` (+1),
  eced4bb6a0234c9e A213 `FDIST(TIME(0,11,(Data!A2/0.2)),1,8)` (-1): the bpser branches on F's w5 arguments.
- ironcalc-calc BETADIST_BETAINV.xlsx Sheet1!H15 `BETA.INV(G15,...)` Excel 0.7, ours 0.6999999999999997 (-2 ulp):
  bpser(a0, 1, x0) at a whole b0 now takes L = -LN(a0) at every a0 (T-2847), which makes BINOM.DIST's whole shapes
  exact (ironcalc-calc BINOM.xlsx E5:E8, E75) but moves BETA.DIST(0.5,2,2,TRUE) (G15) 1 -> 2 ulp and this chained
  BETA.INV with it (docs/research/t2506/RESULTS.md).

## Acceptance criteria
- [ ] Each cell above is bit-exact or named in the Outcome with the form tested and its ulp; no other cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The seams, harness and per-branch counts (T-2977).
