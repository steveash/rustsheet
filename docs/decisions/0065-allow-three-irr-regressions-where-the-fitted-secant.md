# 0065 — Allow three IRR regressions where the fitted secant replaces Newton

- **Status:** accepted (2026-10-16, T-0916)
- **Context:**
  - T-0916 replaces IRR's Newton-plus-bisection with the secant in v = 1/(1 + r) fitted by T-1533
    (financial.md "IRR is a secant in v"). On the oracle fit set the engine goes from 1 to 189 of
    263 cells bit-exact (207 within 4 ulp), the lab model's exact count. In the gated suites the IRR
    part fixes 101 cells (53 ironcalc-calc, 48 probes); XIRR's bisection adds 197 with no regression.
  - Three cells fail now that matched before. Each is among the cells the fitted model does not
    reproduce (74 of the fit set's 263 are open). The old Newton iteration happened to stop on
    Excel's last digit there:
    - `ironcalc-calc/FINANCIAL/IRR.xlsx` General!L42: `IRR(A42:F42)`, Excel 0.5103246911340793,
      now 0.5103246911340791;
    - `probes/fin-argument-modes` Sheet1!H374 and H375: `IRR(L1:L8)` and
      `IRR({-10000,"2750",4250,3250,2750})`, Excel 0.013442343703144344, now
      0.013442343703144566.
- **Options considered:**
  - *Keep Newton for IRR, ship only XIRR.* This loses 101 gated fixes and keeps an algorithm the
    oracle rejects (1/263).
  - *Special-case the three cells.* That fits to individual cells, which decision 0056 forbids.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-0916 PR, for
  exactly these three cells. They join the IRR remainder that T-0916's Outcome lists (the model's
  last bit: NPV's evaluation order).
- **Consequences:** `eval/baseline/ironcalc-calc.txt` and `eval/baseline/probes.txt` gain these
  three lines and lose 298. Reviewers confirm the baseline diff adds no other line.
