---
id: T-3382
title: "Incomplete beta L140 bup+bgrat at 1 < b < 2: Excel's density in bup's orientation times x y over b0, plus an accurate bgrat increment (T-2512)"
milestone: M9
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BETA.DIST's cdf (and BETADIST, BETA.INV's search) on bratio's L140 bup+bgrat branch with one bup term
(1 < b0 + 1 < 2 on the bgrat orientation: a > 15, x > 0.7, bratio's lambda >= 0) is within 15 digits of Excel, where
today's kernel is visibly off on almost all of them.

## Context
- T-2512 (docs/research/t2464/r3-bgrat/RESULTS.md findings 1-3; statistics.md "Incomplete beta: the integrated
  model" bullet "L140 bup+bgrat at 1 < b < 2 (T-2512)"): Excel's cell is RN(W + D) with
  W = fl(x87(P · fl(x·y) / b0)), P = the beta density in bup's orientation (BETA.DIST(y, b0, a, FALSE), not
  (x; a, b0)), and D the bgrat increment I_x(a, b0).
- Measured with the engine's own density (`beta_density.rs`) and an accurate increment: 282/2,824/3,042 of 3,072
  (exact/4 ulp/15 digits) against today's 26/113/344 (`r3-bgrat/engdens.py`, `engdens.out`).
- The increment must be accurate, not TOMS bgrat as written. TOMS hands its 15·eps tolerance to grat1, whose
  continued fraction then stops tens of ulps short (13/142/162 of 2,908 on Excel's direct bgrat cells). Use grat1's
  continued fraction in extended precision at eps = 1e-15 with the stored lnx (Excel's LN / log1p at the 0.01 cut)
  and z = fl(-nu·lnx), the rest extended (206/1,672/2,158 against CR 208/1,598/2,091; `bgld.py`), or a double-double
  evaluation. `bratio/bgrat.rs` `bgrat_half` is the b = 1/2 template: extend it, do not copy it (directive 3).
- Cases: probes-numeric t2464-w2f-absweep, t2464-w2f-xruns-1, t2464-w2f-xruns-2 (BETA.DIST cdf, L140BupBgrat), and the
  direct bgrat cells of probe t2512-r3-bgrat-split.
- Code: `functions/statistical/bratio/mod.rs` (`Branch::L140BupBgrat` keeps today's kernel), `bratio/bup.rs`,
  `bratio/bgrat.rs`, `beta_density.rs`.

## Acceptance criteria
- [x] L140 bup+bgrat with one bup term, b0 != 1/2, takes the composition above in `bratio::beta_dist_lower_tail`.
      On the 3,072 cells of the three w2f cases at least 3,000 are within 15 digits (`make eval` per-case deltas in
      the Outcome).
- [x] No probes-numeric or gated cell goes from within 15 digits to visibly off. Any that does is listed with
      Excel / new / exact values, or the branch is narrowed.
- [x] Unit tests pin a few Excel cells from the cases above, with the case and cell named.
- [ ] `make check` green; reviewers approve.

## Out of scope
- n > 1 bup terms (2 <= b < 40) and F, BINOM, NEGBINOM callers: no probe covers them yet (T-3383).
- Excel's D or density to the bit (T-3383, the density's algdiv lnB).

## Outcome
`bratio::beta_dist_lower_tail` (BETA.DIST's cdf; BETA.INV's search crosses it too) takes L140 bup+bgrat with one bup
term as RN(W + D): `bup.rs` `l140_bup_bgrat`, W = fl(x87(P·fl(x0·y0)/b0')) from the engine's density in bup's
orientation, D from `bgrat.rs` `bgrat_increment`, TOMS bgrat at 0 < b < 1 in double-double with J_0 = Q(b, z)/r by
Legendre's continued fraction (no grat1) and ln Gamma(b) from `ln_gamma_1p_small`. `bgrat_half` now shares that core
(`bgrat_dd`, `algdiv_small`, `asymptotic_sum` take b; at b = 1/2 the arithmetic is unchanged and its tests pass bit for bit).
Eval, the 3,070 L140 cdf cells of t2464-w2f-absweep, -xruns-1 and -xruns-2 that change: bit-exact 26 -> 278, within
15 digits 342 -> 3,037 (absweep 38 -> 1,008 of 1,023, xruns-1 63 -> 1,006 of 1,024, xruns-2 241 -> 1,023 of 1,023).
Two cells leave 15 digits by under 2 ulp, ours nearer the exact value than Excel's (listed in statistics.md
"L140 bup+bgrat at 1 < b < 2"). No gated cell moved (`make eval-check`: 0 regressions, 0 fixed in every gated suite). No constant is chosen by score,
but W's rounding (one register rounding against two stored ops) and the density's orientation were picked by ranking
candidates on Excel cells, so R4's structural-model guards apply (decision 0188 R4, no `FITTED` tag):
- Hold-out (T-2512's md5-odd split, fixed before scoring; `docs/research/t2464/r3-bgrat/bupdens.out`): register form,
  bup orientation 1,306/1,613/1,613 (tune 1,209/1,459) against two stored ops 1,046 and the caller's orientation 672.
  The margin was not named before scoring.
- Fresh sweep after the freeze: probes-numeric t3382-l140-sweep (seed 33820, 300 points log-uniform over the whole branch,
  a0 in (15, 1E4], 1 < b0 < 2, x0 > 0.7, z in [0.5, 700], half swapped; generator `docs/research/t3382/gen_sweep.py`),
  excel-graph. Exact / within 4 ulp / within 15 digits: main 75/108/121, HEAD 163/271/295. One cell leaves 15 digits
  (A36, listed in statistics.md); the 5 still visibly off are A6 (z > 650, today's kernel, unchanged) and A36, A76,
  A208, A251, each nearer Excel than before except A36.
- Gate A envelope (no BETA.DIST table, so on the sweep against 50-digit mpmath `betainc`, as T-3443 did): all 300 cells
  inside (within 15 digits of Excel or within 1E-9 of the math); the largest relative error against the math is 4.1E-13
  for Excel and for ours. Follow-ups: n > 1 terms stay T-3383's; T-3766 folds `large_shape_beta.rs`'s double fraction and series onto
this core (quality review). Also carries T-3753 (filed from #2769's review on this branch by the earlier run; a task file only,
kept here so it is not lost).

Every other case checked against the branch's base (`rustsheet-eval show --all` on both binaries, the 375 cases of
every suite whose formulas name BETA or a Student t function): besides the three w2f cases only probes
stat-quantile-lift Sheet1!E9 (`BETA.INV(C1,1.0024017013258346,12177.162026553196)`) moves, from 0.0030131411834558195
to 0.003013141183455823 against Excel's 0.0691959487211844: visibly off before and after (its search crosses the
new cdf). On the three w2f cases as wholes, within 15 digits: absweep 7,198 -> 8,168, xruns-1 14,899 -> 15,842, xruns-2
15,475 -> 16,257; bit-exact 3,049 -> 3,309, 7,079 -> 7,070 (nine cells leave the bit, all but A8909 still within 15
digits), 7,619 -> 7,620. The only cells that leave 15 digits are the two listed in statistics.md.
