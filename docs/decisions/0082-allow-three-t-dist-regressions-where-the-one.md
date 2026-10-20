# 0082 — Allow three T.DIST regressions where the one-tail Student-t model replaces statrs

- **Status:** accepted (2026-10-20, T-0969)
- **Context:**
  - T-0969 replaces statrs' Student-t CDF (`1 - cdf` for every upper tail) with one tail
    L = P(T > |t|) from which T.DIST, T.DIST.RT, T.DIST.2T and T.TEST follow (statistics.md "T.DIST.RT
    and T.DIST.2T are not 1 - T.DIST"): exact double-double values away from the deep tail, Cephes'
    power series in it, and a double-double density for integer df up to 1000. In the gated suites it
    fixes 27 `ironcalc-calc` cells; on the probes-numeric t sweep T.DIST goes from 78 to 118 of 150
    bit-exact, T.DIST.RT from 34 to 73, T.DIST.2T from 24 to 51, the density from 9 to 38.
  - Three cells fail now that matched before. Each is a last digit where statrs' noise happened to
    land on Excel's:
    - `probes/stat-argument-modes` Sheet1!H505: `T.DIST(1.5,5,TRUE)`, Excel 0.9030481598787632, now
      0.9030481598787633 (the correctly rounded CDF and 1 - L both give ...633);
    - `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST.2T T.INV.2T`!D15 and D37:
      `T.INV.2T(T.DIST.2T(1,1),1)`, Excel 0.9999999999999996, now 1.0000000000000016 (NumericClose).
      C15/C37 (`T.DIST.2T(1,1)`) now pass with Excel's 0.5000000000000001; main's C15 was
      0.5000000000000004 and its T.INV.2T (T-0967) happened to land on Excel's D15.
  - Ungated suites (not in the baseline gate; disclosed here): probes-numeric fixes 168 cells and
    newly misses 12 by one ulp where statrs had matched (among them `T.DIST.RT(2,1)`,
    `T.DIST.2T(2,1)`, `T.DIST(-2,1,TRUE)`, `T.DIST(0.1,2,TRUE)`, `T.DIST(10,10,FALSE)`,
    `T.DIST(-10,10,FALSE)` and their m9-t-nonint-df twins at df 1.5 and 1.999). Two further cells
    the review found (`T.DIST.RT(1E-10,3)` = 0.5 and gen-m9/3ec66461400faf2f A15 = 0) are kept by
    modelling them (statistics.md: n + t^2 rounding to n; the subnormal flush kept from main).
- **Options considered:**
  - *Keep statrs where it matched.* That fits individual cells, which decision 0056 forbids.
  - *Wait for Excel's tail noise.* No candidate reproduces it yet (T-0969 attempts 1 to 3).
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-0969 PR, for
  exactly these three cells.
- **Consequences:** `eval/baseline/probes.txt` gains one line and `eval/baseline/ironcalc-calc.txt`
  gains two and loses 27. Reviewers confirm the baseline diff adds no other line. D15/D37 can come
  back with T-0967's inverse or a model of Excel's tail noise.
