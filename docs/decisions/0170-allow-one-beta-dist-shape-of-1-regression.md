# 0170 — Allow one BETA.DIST shape-of-1 regression where Excel's expm1 and log1p take w

- **Status:** accepted (2026-10-30, T-3170)
- **Context:**
  - At a caller's shape of 1 outside Excel's closed forms (bratio's oriented a0 = 1, y0 > 0.9), the engine computes
    w = -expm1(b0·ln1p(-x0)) and w1 = (0.5 - w) + 0.5 (`incomplete_beta.rs::shape_one_tails`, T-3007). T-3170 scored
    eight forms of w over every suite (statistics.md "Incomplete beta at a shape of 1 outside the closed forms"). With
    Excel's own expm1 (Kahan's form, T-2240) and Excel's own log1p (the stored atanh series, T-2464) in place of libm's,
    71 cells become bit-exact (gated: 6 gen-m9, 3 probes; 62 probes-numeric) and 6 move off: four by 1 ulp,
    probes-numeric t2220-f-dist-sweep-rt A195 by 4 ulp, and probes-numeric t1745-beta-inv-roundtrip G145
    (`BETA.DIST` at Excel's own BETA.INV(0.05,...) quantile, Excel 0.04999999999999938, now 0.04999999999999949) by
    16 ulp, which makes it visibly off. A 1-ulp step of w near 0.95 is 16 ulp of the small tail w1 = (0.5 - w) + 0.5
    there. No cell stops being visibly off, so decision 0083's standing allowance (conditions 1 and 3) does not cover
    the trade.
  - One of the 6 is gated: `probes/ftest-decomposition` Calc!B12, BETA.DIST at x formed by the sheet with shapes
    (3.5, 1). Excel 0.7381591796246966, now 0.7381591796246965 (1 ulp). The same sheet's F.DIST.RT at the same point
    (B5, 0.7381591796246967) stays bit-exact, and its F.DIST (B4), F.DIST.RT (B6) and two-tailed F (B8) become
    bit-exact.
- **Options considered:**
  - *Keep libm's expm1 and log1p.* That keeps B12 and G145 and gives up 71 bit-exact cells, 9 of them gated.
  - *Pick the form by caller (BETA.DIST against F) or by cell.* Neither has evidence behind it, and it would fit
    individual cells, which decision 0056 forbids. The other forms scored that keep B12 (libm's expm1, or LN of the
    caller's y0) lose more cells than they win.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-3170 PR, for exactly this cell
  (1 ulp, not visibly off). The ungated G145 and the other 4 cells need no baseline line; all 6 stay with T-3192.
- **Consequences:** `eval/baseline/probes.txt` gains this one line and loses 3; `eval/baseline/gen-m9.txt` loses 6.
  Reviewers confirm the baseline diff adds no other line.
