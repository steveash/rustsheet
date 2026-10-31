---
id: T-3451
title: "Oracle sweep: BETA.DIST and BETA.INV at shapes a = b from 1E-5 to 1E-2 around p = 0.5, for BETADIST_BETAINV H26"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel values for BETA.DIST (cumulative) and BETA.INV at tiny equal shapes are in `eval/`, enough for T-3452 to fit
and hold out how Excel's BETA.INV lands where its cdf is nearly flat.

## Context
Gap triage 2026-10-31e. ironcalc-calc/STATISTICAL/BETADIST_BETAINV.xlsx H26 `BETA.INV(G26,0.0001,0.0001,0.15,1.2)` is
0.2339999999999406 in Excel and 0.23400000000265136 ours, while G26 `BETA.DIST(0.234,0.0001,0.0001,TRUE,0.15,1.2)` is
within 15 digits (Excel 0.4998779049300665, ours 0.49987790493006656, mpmath 0.49987790493006460733: both off the math,
Excel by about 13 ulp). At a = b = 1E-4 the cdf is almost flat around p = 0.5, so a few ulp in the cdf move the
quantile by 1E-11. The cached evidence has only three x points per shape at a = 1E-4 (probes-numeric/m9-beta-dist-a0p0001)
and no BETA.INV at tiny equal shapes, too few to fit Excel's cdf or inverse there.

## Acceptance criteria
- [ ] Two new probes-numeric cases (milestone M9), filled by the excel-graph oracle: a fit set and a hold-out set of
      at least 200 points each, drawn with a recorded seed. They hold BETA.DIST(x,a,a,TRUE) and BETA.INV(p,a,a) with a
      log-uniform in [1E-5, 1E-2], x in [0.01, 0.99] and p in [0.49, 0.51] (half the points within 1E-3 of 0.5), plus
      the round trip `BETA.INV(BETA.DIST(x,a,a,TRUE),a,a)` and BETA.DIST with a different second shape b in
      [1E-5, 1E-2] on a quarter of the rows.
- [ ] One more probes-numeric case, from a different seed, is filled and left unread for T-3452's fresh-sweep check
      (decision 0188 R4).
- [ ] statistics.md records what the sweep shows (whether Excel's BETA.INV is its cdf's crossing at tiny shapes),
      citing cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (T-3452).
