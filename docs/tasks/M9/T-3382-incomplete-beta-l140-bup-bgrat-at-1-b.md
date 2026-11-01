---
id: T-3382
title: "Incomplete beta L140 bup+bgrat at 1 < b < 2: Excel's density in bup's orientation times x y over b0, plus an accurate bgrat increment (T-2512)"
milestone: M9
lane: engine
status: todo
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
- [ ] L140 bup+bgrat with one bup term, b0 != 1/2, takes the composition above in `bratio::beta_dist_lower_tail`.
      On the 3,072 cells of the three w2f cases at least 3,000 are within 15 digits (`make eval` per-case deltas in
      the Outcome).
- [ ] No probes-numeric or gated cell goes from within 15 digits to visibly off. Any that does is listed with
      Excel / new / exact values, or the branch is narrowed.
- [ ] Unit tests pin a few Excel cells from the cases above, with the case and cell named.
- [ ] `make check` green; reviewers approve.

## Out of scope
- n > 1 bup terms (2 <= b < 40) and F, BINOM, NEGBINOM callers: no probe covers them yet (T-3383).
- Excel's D or density to the bit (T-3383, the density's algdiv lnB).
