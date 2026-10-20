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
      `T.INV.2T(T.DIST.2T(1,1),1)`, Excel 0.9999999999999996. T.DIST.2T(1,1) is now the exact 0.5
      (Excel 0.5000000000000001, C15/C37 stay NumericUlp), and T.INV.2T (T-0967) inverts that.
- **Options considered:**
  - *Keep statrs where it matched.* That fits individual cells, which decision 0056 forbids.
  - *Wait for Excel's tail noise.* No candidate reproduces it yet (T-0969 attempts 1 to 3).
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-0969 PR, for
  exactly these three cells.
- **Consequences:** `eval/baseline/probes.txt` gains one line and `eval/baseline/ironcalc-calc.txt`
  gains two and loses 27. Reviewers confirm the baseline diff adds no other line. D15/D37 can come
  back with T-0967's inverse or a model of Excel's tail noise.
