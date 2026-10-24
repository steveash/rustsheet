# 0106 — Allow one 5-ulp F.TEST regression from the T-2361 kernel fit

- **Status:** accepted (2026-10-24, T-2361)
- **Context:**
  - T-2361 replaces F.TEST's tail (f = v1/v2, twice the smaller of `f_cdf` and `f_sf`) with the
    fitted form of docs/excel-semantics/statistics.md "F.TEST / FTEST: larger variance on top":
    the larger variance over the smaller, then exp(a ln x) exp(b ln y) times a remainder summed in
    double-double. In the gated suites it fixes 61 cells (55 gen-m9, 4 probes, 2 ironcalc-calc),
    and the numerically visibly-off F.TEST cells of gen-m9, probes and ironcalc-calc go from 37 to
    10 (owned by T-2361: 21 to 8), including probes/stat-argument-modes H248:H251.
  - 17 cells that matched before now differ, none visibly. 16 are within 4 ulp and fall under
    decision 0083's standing ulp-regression allowance. One is 5 ulp, which the allowance does not
    cover: `gen-m9/3e98b26cf6713e77` Calc!A8, `FTEST(Data!A10:D11,Data!B2:D10)` (df 9 and 4),
    Excel 0.10380574639921504, now 0.10380574639921511. Excel itself is 5 ulp below the exact
    two-tailed p there (0.1038057463992151); the old kernel's continued-fraction rounding happened
    to land on Excel's last digit.
- **Options considered:**
  - *Keep the old kernel.* It leaves 27 more visibly-off cells, the named probe cells among them.
  - *Special-case the cell.* That fits one cell, not an algorithm (decision 0056: fits are scored over a sweep).
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-2361 PR for
  the 17 cells T-2361's Outcome lists, this 5-ulp cell included. Follow-up T-2571 owns them.
- **Consequences:** `eval/baseline/gen-m9.txt`, `probes.txt` and `ironcalc-calc.txt` gain exactly
  these 17 lines and lose 61. Reviewers confirm the baseline diff adds no other line.
