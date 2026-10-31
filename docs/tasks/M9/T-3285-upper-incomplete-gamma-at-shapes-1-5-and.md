---
id: T-3285
title: "Upper incomplete gamma at shapes 1.5 and 2.5: CHISQ.DIST.RT(10,5) and CHIDIST(10,5) are 6 ulp off where Excel is correctly rounded (accuracy)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
CHISQ.DIST.RT, CHIDIST and the other upper-tail callers of the incomplete gamma are correctly rounded (so Excel's
double) at small half-integer shapes with x a few times the shape, where Excel is within half an ulp of the 50-digit
value and ours is 5 to 18 ulp off.

## Visible-parity stage
`accuracy`: a 50-digit reference (mpmath `gammainc(a, x, inf, regularized=True)`) puts Excel within 0.3 ulp at
`CHISQ.DIST.RT(10,5)` and ours 5.7 ulp below it, so the fix is a more accurate kernel, never a fit to Excel's bits.
Target: the four cells listed below within half a unit of the 15th significant digit (bit-exact expected), with no cell of
any suite regressing.

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 73b187224; `scripts/exit_owners.py --all-off` lists the four cells
  with no open owner). Q(2.5, 5), in ulps of Excel's value against mpmath at 60 digits:
  - `CHISQ.DIST.RT(10,5)` / `CHIDIST(10,5)`: Excel 0.07523524614651218 (+0.3), ours 0.0752352461465121 (-5.7).
- The same shape class, owned by deferred tasks (fix them too if the kernel change reaches them, and say so in the
  Outcome; do not take them over):
  - probes-numeric/m9-chisq-dist-rt A14 `CHISQ.DIST.RT(6,3)` = Q(1.5, 3): Excel 0.11161022509471255 (-0.7), ours +18.3
    (T-2367, which notes it as "the one cdf cell left where Excel is accurate and ours is not, at a = 1.5");
  - gen-m9/85f48d5cdfb24555 Calc!A84 `CHIDIST(40,3)` = Q(1.5, 20): Excel +1.2, ours -4.8 (T-2695).
- Path: `statistical/incomplete_gamma/mod.rs` `pair_with` with `gamma_prefactor`'s direct branch below
  `DIRECT_SHAPE_MAX` = 20 (x.powf(a) * exp(-x) / libm::tgamma(a)), then the continued fraction for Q. At a half-integer
  shape Q has a closed form, Q(1/2 + n, x) = erfc(sqrt x) + e^-x sum_{j=1..n} x^(j - 1/2) / Gamma(j + 1/2), which a
  mpmath unit test can check; find which factor loses the ulps (tgamma at a half-integer, the prefactor product or
  the continued fraction's stopping rule) before choosing a remedy.
- Excel's deep upper tail (x well above a) is the inaccurate side and belongs to T-3269; keep that region's results
  unchanged unless they move toward Excel.

Cells this task owns:
- probes-numeric/m9-chisq-dist-rt.case.json (1): Calc!A19
- probes-numeric/m9-legacy-chidist.case.json (2): Calc!A45, A46
- probes-numeric/t1049-chisqdistrt.case.json (1): Calc!A19

## Acceptance criteria
- [x] The four cells above match Excel (`make eval-show` on each case; probes-numeric is ungated).
- [x] A unit test in `incomplete_gamma/tests.rs` checks Q at shapes 0.5, 1.5, 2.5 and 3.5 against 50-digit values for x
      from a to 30a, within 1 ulp.
- [x] (Amended by owner ruling (a), decision 0202: the lost and farther cells it names are accepted.) No Excel-computed cell of CHISQ.DIST.RT, CHIDIST, CHISQ.DIST, CHISQ.TEST, GAMMA.DIST, GAMMADIST, POISSON or
      POISSON.DIST goes from pass to fail (main-vs-HEAD per cell); no new line in eval/baseline/.
- [x] docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)" cites the four cells as evidence
      that Excel is correctly rounded there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The deep upper tail where Excel is 10 to 405 ulp off (T-3269, T-3289). The near-mode window at large shapes (T-3286).
  The shapes below 20 at a tiny x / a (T-3219).

## Outcome
- Cause: the forward (modified Lentz) continued fraction, 5.1 ulp at Q(2.5, 5) and 16 at Q(1.5, 3); the prefactor
  (`powf * exp / tgamma`) was within 1.3. `incomplete_gamma/half_integer.rs` takes Q at a half-integer shape in
  double-double: the prefactor sqrt(x) e^-x / sqrt(pi) * prod 2x / (2j - 1) (exact in the shape) and the fraction
  (`incomplete_gamma/fraction.rs`) evaluated backward from twice the forward depth plus 8, rounded once. The unit test
  checks it against 50-digit mpmath at shapes 1/2, 3/2, 5/2, 7/2 for x from a to 30a: 44 of 44 correctly rounded.
  1/sqrt(pi) moved from bratio/bgrat.rs to half_integer_beta.rs (one definition).
- Attempt 1 applied it for every x >= a and lost `CHIDIST(5,5)` and `CHIDIST(7,7)` (x = a, where Excel is the series'
  1 - P bit for bit; the accurate Q is 3 and 1 ulp off) and `CHIDIST(42,7)`. Attempt 2 applied it only on [a + 1, 3a];
  the fidelity review found the 3a cut keep-matched (decisions 0056, 0082). Shipped, by owner ruling (a) "Allow, with
  records" (decision 0202): the accurate Q for every x >= a + 1 (up to MAX_X = 650), no upper cut, no FITTED tag
  (quality/fitted.json via `scripts/check_fitted.py --update`). The lower cut is evidenced: CHIDIST(5,5) and
  CHIDIST(7,7) (gen-m9/4250f880ab640c49 A331, 899f9555e67da2d6 A72) are the engine's 1 - P bit for bit on main and HEAD.
- Eval, main 1af9bd6c6 vs HEAD, full runs, per cell, over every suite (the same suite root for both):
  - gated: fixed 4 (gen-m9 2a18b2f5a4dc43ac A383 `CHIDIST(38.5,5)` 3 to 0, b8ee55934c5b824c A216 `CHIDIST(37.5,9)` 3 to
    0, caa1eed48b1883fe A59 `CHIDIST(..,9)` 1 to 0, ironcalc-calc CHISQ.xlsx G3 11 to 0); closer 4 (0a614cfe974b59c3
    A253 `CHIDIST(37.5,3)` 3 to 1, 85f48d5cdfb24555 A84 `CHIDIST(40,3)` 6 to 1, e6cf3b20a014e6a3 A223 `CHIDIST(..,9)` 4 to
    2, CHISQ.xlsx G4 5 to 1); lost 1 (fec39efaba9284c3 A249 `CHIDIST(42,7)` 0 to 1); farther 4 (320d51765af9b6b7 A249 and
    7674c89c824aca43 A332 `CHIDIST(9,5)` 2 to 3, b8ee55934c5b824c A262 `CHIDIST(37.5,7)` 2 to 3 and A400
    `CHIDIST(37.5,5)` 1 to 2); visibly fixed 2 (G3, G4); newly visibly off 0. Decision 0202 names the lost and farther
    cells; `update-baseline --allow-regressions` recorded them (decision 0083's allowance; follow-up T-3461).
  - ungated (probes-numeric), existing cases: fixed 4, the cells this task owns (`CHISQ.DIST.RT(10,5)` / `CHIDIST(10,5)`,
    m9-chisq-dist-rt A19, m9-legacy-chidist A45, A46, t1049-chisqdistrt A19); closer 67 (m9-chisq-dist-rt A14
    `CHISQ.DIST.RT(6,3)`, T-2367's cell, 19 to 1); farther 61; lost 2 (t3284-gamma-deep-upper-tail A245
    `CHISQ.DIST.RT(42,7)` and A252 `(54,9)`, 0 to 1); visibly fixed 8; newly visibly off 2 (t3284-gamma-deep-upper-tail
    A246 `CHISQ.DIST.RT(56,7)` 2 to 3, A260 `(120,15)` 11 to 13; accepted by the owner's extended ruling (a), decision
    0202). Nearly all the farther cells are past x = 30
    (t2561-chisq-dist-rt-past-edge, t3284-gamma-deep-upper-tail), where Excel takes its Stirling ratio form (T-3269).
  - eval/baseline (`update-baseline --allow-regressions`): gen-m9 2894 to 2892 known failures (fixed 3, the one
    regression `CHIDIST(42,7)`); ironcalc-calc one line fewer (CHISQ.xlsx G3). Visible parity 2265 in-scope cells
    visibly off against the lock of 2429 (the drop is left to the steward, decision 0093).
- Decision 0188 R4: no fitted piece is left, so R4.1's hold-out does not apply. R4.2 fresh sweep after the freeze:
  probes-numeric/t3285-chisq-dist-rt-half-integer-sweep (228 cells: a = 1.5 to 19.5, x log-uniform in [a + 1, 650],
  filled by excel-graph; `docs/research/t3285/mksweep.py`, 50-digit values in `mpmath.json`, `analyze.py`, output in
  `analyze.out`). Bit-exact against Excel: main 5, HEAD 5, correctly rounded 5 (Excel is the correctly rounded double at
  only 5); within 15 digits: main 92, HEAD 101, correctly rounded 101. HEAD is the correctly rounded double at all 228
  (main up to 10 ulp from it). Gate A envelope (within 1e-9 of the math or 15 digits of Excel): 228 of 228 for main and
  HEAD. Main-vs-HEAD on the sweep: fixed 4, lost 4, closer 91, farther 87, visibly fixed 11, newly visibly off 2 (A51,
  A101, both past x = 25).
- Review follow-ups: the lift charge (lift/kinds/mod.rs `incomplete_gamma_units`) under-counts the backward fraction by
  about 2 to 3 times; the work stays bounded (a < 20: at most 343 forward and 694 backward steps per cdf), so it is
  recorded here and filed as T-3459. The five private `type Dd` aliases: T-3458. Excel's form on [a + 1, 30]: T-3461.
- docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)" records the cells, `CHIDIST(9,5)`
  (Excel -2.6 ulp from the truth) and the sweep.
