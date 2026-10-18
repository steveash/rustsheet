# 0064 — Allow eight PV and NPER regressions unmasked by T-0901

- **Status:** accepted (2026-10-16, T-0901)
- **Context:**
  - T-0901 implements the fitted PV, FV and NPER formulations (financial.md "Annuity evaluation
    order"). They match all 308 PV, 308 FV and 132 NPER cells of the excel-graph sweeps
    (probes-numeric/m9-fin-pv-fv, m9-fin-nper-npv). In the gated suites they fix 35 ironcalc-calc,
    8 poi and 1 probes cells.
  - Eight other cells passed before only by accident. Each is a PV or NPER in
    `ironcalc-calc/FINANCIAL/PMT.xlsx` whose pmt argument is a PMT cell, and every one of those
    PMT cells is already a known failure (F12, G12, G16, F32, G32, G34, F35). The old PV/NPER
    formulas happened to cancel the engine's PMT error.
  - Given Excel's cached PMT values as inputs, the fitted formulas reproduce all eight cached
    results bit for bit. A Python check through the fit lab's `integer_power` gives I16
    12.199999999999994, I32 and I34 3.000000000000001, J12, K12 and M12 99.99999999999999, J32
    -199.99999999999994 and J35 -200.0.
  - The cells: `ironcalc-calc/FINANCIAL/PMT.xlsx` Sheet1!I16, I32, I34 (NPER) and J12, J32, J35,
    K12, M12 (PV).
- **Options considered:**
  - *Keep the old PV/NPER formulas.* This loses 44 gated fixes and contradicts 748 oracle cells.
  - *Fix PMT in the same PR.* PMT's last digits are unexplained, and that work is time-boxed
    (T-1528, decision 0056). The accurate PMT trades 30 cached fixes for 22 regressions.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-0901 PR,
  for exactly these eight cells. They join T-1528's list: they pass when PMT matches Excel.
- **Consequences:** `eval/baseline/ironcalc-calc.txt` gains these eight lines and loses the
  fixed ones. Reviewers confirm the baseline diff adds no other line.
