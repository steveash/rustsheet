---
id: T-3580
title: "F cdf and tails: Excel's r >= 1 incomplete-beta arguments, and bgrat at b = 1/2 for a df of 1"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
F.DIST(., TRUE), F.DIST.RT and FDIST hand the incomplete beta Excel's own (x, y), and F with a numerator or
denominator df of 1 takes bratio's bgrat branches at b = 1/2 as the Student t tails and BETA.DIST already do.

## Context
- Found by T-3567 (docs/research/t2464/r5-bgrat-fdf1/RESULTS.md; statistics.md "F's incomplete-beta arguments
  keep r >= 1"). With mx = fl(m·x) and the truncated dfs m, n, Excel forms r = fl(n/mx) when mx <= n, else
  r = fl(mx/n), so r >= 1, and passes the smaller argument as fl(1/fl(1 + r)) and the larger as fl(r/fl(1 + r)).
  Today `f_beta_arguments` (crates/rustsheet-engine/src/functions/statistical/fisher.rs) forms both as 1/(1 + .),
  which is right for the smaller one only.
- With those arguments, the T-2509 bgrat at b = 1/2 (`bratio::t_kernel_tails`' `half_bgrat`) fits F at a df of 1:
  the reason T-2510 kept F off it (`FDIST(2,1,9)` 7 ulp, the d1-1 / d2-1 sweep cells visibly off) was the
  argument form, not the kernel. `ported_tails_with` carries `TODO(T-3567)` for this.
- Research model (python, r2-tdist FINAL kernel with `bg = "ld"`): on the hold-out probe
  probes-numeric/t3567-r5-fdf1-hold (3,578 cells) it scores 2,787/3,419/3,463 (bit-exact / within 4 ulp / within 15
  digits) against the engine's 1,857/2,843/3,025; the d1 = 1 bgrat region 991/1,000/1,000 against 298/572/659.
  On cells.tsv's 3,435 F cdf/tail cells the r >= 1 form beats n + mx in every branch, and the a0 = 1 / b0 = 1
  closed forms are 162/162 and 201/201 bit-exact.
- Where the engine's other F pieces (bpser, bup, bfrac) differ from the research kernel, keep whichever is closer
  to Excel per branch, measured on the hold-out; the argument form alone should not be assumed to help there.
- F.INV, F.INV.RT and FINV invert `f_quantile_tail` on today's kernel (T-2511, T-2977): out of scope here unless the
  gated quantile cells stay put.

## Acceptance criteria
- [ ] F cdf and tails use the r >= 1 arguments; F at a df of 1 takes bgrat at b = 1/2.
- [ ] On t3567-r5-fdf1-hold, the bg1 region (d1 = 1, bgrat) reaches at least 990 of 1,000 cells bit-exact, and the
      case as a whole has no cell newly visibly off.
- [ ] `FDIST(2,1,9)` (gen-m9/c18dbf21f11093f3 A151) and the T-2977 gated cells stay at Excel's value; no gated
      known failure appears (`make eval-baseline` only shrinks baselines).
- [ ] statistics.md's "F's incomplete-beta arguments keep r >= 1" entry gets its "Ours:" line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.DIST(., FALSE) (the density, T-2586).
- The bgrat increment at b != 1/2 (T-3582).
