# 0141 — F at (2.5, 2.5, x = 0.6000000000000001) takes the bratio port's value, 5 ulp from Excel

- **Status:** accepted (2026-10-28, T-2847).
- **Context:**
  - T-2847 moves F.DIST, F.DIST.RT and FDIST onto the bratio port's bpser and L140 branches (SPEC-0025's
    caller table, `campaign: accuracy`, decision 0114: a more accurate kernel, never fitted to Excel's last
    bits). Across the gates it fixes 69 gen-m9 cells, 23 ironcalc-calc cells and 4 probes cells.
  - Two of its regressed cells are past decision 0083's 4-ulp allowance: `F.DIST.RT(1.5,5,5)` and
    `FDIST(1.5,5,5)` (probes/stat-argument-modes H221, H226). Excel gives 0.33360956282916693; the port gives
    0.33360956282916665, 5 ulp lower. Both are within 15 significant digits (0.333609562829167).
  - The point is I_x(2.5, 2.5) at Excel's own argument x = 1/(1 + 5/7.5) = 0.6000000000000001, the L140
    branch (bup(0.5, 2.5, 0.6, 0.4, 2) + bpser(2.5, 0.5, 0.4)). A 40-digit mpmath reference at that double x
    gives the upper tail 0.333609562829166621; its nearest double is 0.3336095628291666. The port is 1 ulp
    above it, Excel 6 ulp above it (at the real-number x = 0.6 the reference is 0.33360956282916676, still
    3 ulp from Excel). The companion
    `F.DIST(1.5,5,5,TRUE)` (H215) moves 2 ulp, inside the allowance.
  - docs/research/t2464/r2-impact/RESULTS.md section 6 hypothesis 1 already found the r2-bratio model 2-4 ulp
    off Excel here; no model of Excel's L140 at b0' = 0.5, n = 2 reproduces it yet.
- **Decision:** T-2847 may record H221 and H226 as known failures (`update-baseline --allow-regressions`)
  together with the regressions decision 0083's allowance covers. Excel's value stays the expected one
  (prime directive 1); this is not a deviation, only an ordering of work: the more accurate kernel lands
  first and Excel's last bits at this point are left to a phase-1b follow-up (decision 0114).
- **Consequences:**
  - The follow-up ulp task T-2847 files lists H215, H221 and H226 with the other regressed cells and the two
    hypotheses of r2-impact section 6; whoever finds Excel's L140 form at b0' = 0.5 fixes them.
  - No other cell may use this record: any further regression past 4 ulp needs its own decision.
