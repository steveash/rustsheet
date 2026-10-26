# 0111 — Owner ruling: T-2119 and T-0971 merge with named cells newly visibly off

- **Status:** accepted (2026-10-25, T-2119 and T-0971; ruled by the owner in session
  https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr). It amends decision 0083's standing allowance for these
  two PRs only.
- **Context:**
  - T-2119 (#1620, BETA.DIST's cdf on the T-0970 incomplete beta) and T-0971 (#1536, the accurate incomplete gamma)
    were both blocked. Their baseline step needs `update-baseline --allow-regressions`, and the autonomous workers'
    permission layer refused it. The owner approved running it.
  - Each gated regression is numeric, at most 4 ulp and not visibly off: 9 for T-2119, 15 for T-0971. Decision
    0083's conditions 1, 3 and 4 hold for them.
  - Condition 2 ("no cell becomes visibly off") does not hold. These cells go from within half a unit of the 15th
    significant digit to past it:
    - T-2119, ungated probes-numeric, all BETA.DIST: t0969-beta-prefactor D85 (18 ulp) and D86 (33 ulp; bit-exact
      before), and t0969-t-beta-link E56 (4 ulp) and F6 (6 ulp).
    - T-0971, gen-m9 cells that were already known failures: 285d87cea1aaa73c A261 and 4ab2ba98d26c8a5e A85
      `CHIINV(0.975,5)` (9 ulp), 984cba67bbe6972b A49 `CHISQ.DIST(1E-10,8,...)` (37), a7bf0d8d81d33ba6 A152
      `GAMMADIST(...,7,...)` (49) and f34f66fb7f81e673 A3 `CHIDIST(...,8)` (7).
  - Both PRs are large net visible-parity gains. Counted over every suite, ungated probes-numeric included, in-scope
    visibly-off cells fall from 11,309 on main to 10,296 (T-2119) and to 10,916 (T-0971). On the T-0971 cells
    checked against a 50-digit mpmath reference (`CHIDIST(100,8)`, `CHIINV(0.975,5)`), ours is correctly rounded
    and Excel is 7-13 ulp off. So Excel's own forms have to be fitted. That is follow-up work, not a reason to keep
    statrs.
- **Decision:** T-2119 and T-0971 may merge with exactly the cells named above newly visibly off. Each of them is
  owned by a `campaign: visible` follow-up task: T-2633 for T-2119's cells, T-2638 for T-0971's. Excel stays the
  spec (directive 1). This ruling accepts a temporary gap; it does not settle the cells.
- **Consequences:**
  - The Excel-fidelity reviewer checks the two PRs' newly visibly-off cells against the list above. A cell not
    listed still needs its own decision record.
  - Decision 0083's condition 2 is unchanged for every other PR.
  - T-2633 and T-2638 carry the four probes-numeric cells and the five gen-m9 cells in their acceptance criteria.
