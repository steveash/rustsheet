# 0152 — Owner ruling: T-0972 merges with five cells newly visibly off

- **Status:** accepted (2026-10-28, T-0972; ruled by the owner in session
  https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr). It amends decision 0083's standing allowance for PR #1767 only,
  as decision 0111 did for T-2119 and T-0971.
- **Context:**
  - T-0972 (#1767) refines GAMMA.INV, CHISQ.INV, CHISQ.INV.RT, GAMMAINV and CHIINV to the crossing of the smaller tail
    of the accurate incomplete gamma (`incomplete_gamma` `gamma_quantile`). It was blocked since 2026-10-25 because the
    workers' permission layer refused `update-baseline --allow-regressions`.
  - Measured on the branch merged with `main` at 3fe509e8 (2026-10-28):
    - gated suites: 28 lines fixed (gen-m9 23, ironcalc-calc 5), and 49 cells regressed, all numeric (38 at 1 ulp, 9 at 2
      and 2 at 3). Decision 0083's conditions 1, 3 and 4 hold for them.
    - across every suite, ungated probes-numeric included, against a `main` build: 90 other cells newly fail but stay
      within the 15th digit, and in-scope visibly-off cells fall from 101,602 to 101,575 (net -27).
  - Condition 2 ("no cell becomes visibly off") does not hold for five cells:
    - gen-m9 ecc87f80077959e0 Calc!A168 `CHISQ.INV.RT(MIN(Data!A9:A12),7)`: Excel 8.383430828608388, ours
      8.383430828608383 (3 ulp; it passed on `main`).
    - gen-m9 146b46e74f2dac3b Calc!A167 `CHISQ.INV(DOLLARDE(0.1,28),2)`: Excel 0.8836655045580786, ours
      0.8836655045580779 (6 ulp; already a known failure).
    - ironcalc-calc STATISTICAL/GAMMA.xlsx `GAMMA.DIST GAMMA.INV!F21` `GAMMA.INV(E21,B21,C21)`: Excel 9.999999999999998,
      ours 10.000000000000004 (3 ulp; already a known failure).
    - probes-numeric m9-chisq-inv Calc!A27 and t1049-chisqinv Calc!A19 `CHISQ.INV(0.999,5)`: Excel 20.515005652432812, ours
      20.515005652432876 (18 ulp; already failing). Here Excel's double cdf is flat and Excel is 40 to 100 ulp from the
      true quantile (statistics.md, the T-0972 bullet).
- **Decision:** T-0972 (#1767) may merge with exactly these five cells newly visibly off. Follow-up T-3016 owns them
  and the 49 gated ulp cells. Excel stays the spec (directive 1): this accepts a temporary gap and settles no cell.
- **Consequences:**
  - The Excel-fidelity reviewer checks #1767's newly visibly-off cells against this list; any other cell needs its own
    decision record.
  - Decision 0083's condition 2 is unchanged for every other PR.
