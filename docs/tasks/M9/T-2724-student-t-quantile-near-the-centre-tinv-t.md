---
id: T-2724
title: "Student-t quantile near the centre: TINV, T.INV.2T and CONFIDENCE.T at alpha near 1 lose the small quantile to cancellation (accuracy, split from T-0967)"
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
TINV, T.INV.2T and CONFIDENCE.T with a probability near 1 (and T.INV with p near 0.5), whose quantile is a tiny t near
the centre of the distribution, are within half a unit of the 15th significant digit of Excel's value instead of
losing most of their digits to cancellation.

## Context
Gap triage 2026-10-25 (eval on origin/main becefc6f3). The two-tailed path asks `t_upper_quantile(dist, df, p/2)`,
which for q = p/2 near 0.5 calls `t_quantile(1 - q)` and then `t_inverse_cdf` (functions/statistical/inverse_beta.rs):
it inverts `I_y(df/2, 1/2) = 2(1 - p')` for y close to 1 and back-transforms with `sqrt(df (1 - y) / y)`, so `1 - y`
keeps only the digits of y below 1. The results are off by 1E-5 relative at alpha 0.999999, by 100x at 1-1E-10 and are
exactly 0 at 1-1E-16; CONFIDENCE.T (functions/statistical/normal.rs `fn_confidence_t`) inherits it. Examples (excel-graph):
- gen-m9/b9a32ce9a1fd0d71 Calc!A58 `=TINV(1-1E-10,5)`: Excel 1.3171528710518458e-10, ours 2.356080457693621e-08
- gen-m9/cb61169f4c033d4b Calc!A71 `=TINV(1-1E-16,6)`: Excel 1.4503892858778844e-16, ours 0
- gen-m9/334f3dc9ca937f10 Calc!A128 `=T.INV.2T(0.999999,7)`: Excel 1.2987301378605883e-06, ours 1.2986257252981012e-06
- gen-m9/e6cf3b20a014e6a3 Calc!A90 `=CONFIDENCE.T(1-1E-10,0.999,13)`: Excel 3.545611944164206e-11, ours 1.0113222215567095e-08
- probes-numeric/m9-t-inv-2t Calc!A117 `=T.INV.2T(0.9999999999,1000)`: Excel 1.2536276086925312e-10, ours 3.332000937312528e-07
- ironcalc-calc T_DIST.xlsx `T.DIST T.INV T.DIST.RT`!E20 `=T.INV(T.DIST(1E-6,11),11)`: Excel 9.999999999666944e-07, ours 1.0000999564468016e-06

Excel is accurate here: an mpmath check (50 digits, bisection on `betainc(df/2, 1/2, 0, df/(df+t^2))` at the double
value of p, triage scratch script) gives 1.3171528710518443e-10 for `TINV(1-1E-10,5)`, 1.41421367938565e-10 for
`TINV(1-1E-10,2)`, 1.3063945295223615e-06 for `TINV(0.999999,6)` and 1.4503892858778863e-16 for `TINV(1-1E-16,6)`:
Excel is within 1.6E-15 relative of the true quantile in every case checked, so this is the `accuracy` stage (decision
0114): a kernel that does not cancel, never a fit to Excel's bits. One way: for two-tailed p >= 1/2, `c = 1 - p` is
exact (Sterbenz), and `t^2 = df w / (1 - w)` with `w` solving `I_w(1/2, df/2) = c` keeps the small quantity
small (for tiny c, `I_w(1/2, df/2) ~ 2 sqrt(w) / B(1/2, df/2)` gives the start); T.INV with p near 1/2 is the same
inversion at c = |2p - 1|. Keep the iteration bounded (directive 5; the lift table charges `T_QUANTILE_WEIGHT`).

Split from T-0967 (blocked behind T-0969's time-boxed CDF fit): T-0967's own hypothesis already says "for the 2-tailed
and near-0.5 cases solve on the tail directly (no `1 - p`)", and this part needs no Excel kernel, only accuracy. T-0967
keeps every other cell it lists (the central-region last digits such as `TINV(0.5,9)`, tiny p, CONFIDENCE.T last digits).

Owned cells (62 known-failure lines, 61 visibly off; `make eval-show CASE=<case>`):
- gen-m9/097af1ca33638670.case.json (1): Calc!A101
- gen-m9/09b47629f54b9817.case.json (1): Calc!A40
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A341
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A340
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A389
- gen-m9/334f3dc9ca937f10.case.json (2): Calc!A128, A333
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A5
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A181
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A361
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A70
- gen-m9/44a8841352dc48e9.case.json (1): Calc!A226
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A5, A371
- gen-m9/5389dd4d5b5739d2.case.json (3): Calc!A102, A299, A361
- gen-m9/5395b8a427764a3f.case.json (2): Calc!A37, A397
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A271
- gen-m9/5be29fa316931f05.case.json (1): Calc!A3
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A364
- gen-m9/775d053de498b8c7.case.json (1): Calc!A326
- gen-m9/7f7ca726c3f68554.case.json (1): Calc!A394
- gen-m9/7fd0a98183c6babf.case.json (2): Calc!A356, A394
- gen-m9/928d08e3e1eecb5f.case.json (2): Calc!A201, A226
- gen-m9/984cba67bbe6972b.case.json (2): Calc!A128, A177
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A42
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A64
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A58
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A99
- gen-m9/cb61169f4c033d4b.case.json (1): Calc!A71
- gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A145
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A90
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A327
- ironcalc-calc/STATISTICAL/T_DIST.xlsx (4): sheet `T.DIST.2T T.INV.2T` D20, D62, D63; sheet `T.DIST T.INV T.DIST.RT` E20
- probes-numeric/m9-t-inv-2t.case.json (20): Calc!A103, A104, A116, A117, A12, A129, A13, A130, A25, A26, A38, A39, A51, A52, A64, A65, A77, A78, A90, A91


## Acceptance criteria
- [ ] Every owned cell above is within half a unit of the 15th significant digit of Excel's value (an expected
      value is never matched by 0); the ones that also become bit-exact leave eval/baseline/gen-m9.txt,
      ironcalc-calc.txt and probes-numeric.txt (`make eval-baseline`), and no new known-failure line appears in any
      baseline.
- [ ] Unit tests in t_dist.rs for two-tailed p = 0.999999, 1-1E-10 and 1-1E-16 at df 1, 2, 5, 1000 and 1E6, and for
      T.INV at p = 0.5 +- 1E-10, against the Excel values above; the iteration count is bounded.
- [ ] The rule (Excel's centre quantiles are accurate; ours cancelled) is recorded in docs/excel-semantics/statistics.md
      "Student t sweep", citing these cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's own last digits elsewhere in the family (T-0967), tiny p (T-1542), p in (1,2) (T-2096), p just above 1 (T-2839; a fix there may share this kernel, so coordinate), the T.DIST kernel (T-0969).

## Visible-parity stage
Decision 0083/0114, stage `accuracy`: done when every owned cell is within half a unit of Excel's 15th significant digit.
Cells still not bit-exact then stay with T-0967 (bit level), named in the Outcome.
