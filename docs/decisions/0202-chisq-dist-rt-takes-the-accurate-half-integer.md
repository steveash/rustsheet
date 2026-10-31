# 0202 — CHISQ.DIST.RT takes the accurate half-integer Q for every x >= a + 1 (owner ruling)

- **Status:** accepted (2026-10-31, T-3285; owner ruling (a) "Allow, with records", session
  https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a). Applies decision 0083's standing ulp-regression allowance to
  T-3285's PR, with the cells below named.
- **Context:**
  - T-3285 computes Q(a, x) at a half-integer shape from 3/2 to 20 in double-double, rounded once
    (`incomplete_gamma/half_integer.rs`): correctly rounded against 50-digit mpmath. Excel is correctly rounded at
    `CHISQ.DIST.RT(10,5)` = Q(2.5, 5) and `CHISQ.DIST.RT(8,3.34)` = Q(1.5, 4), the cells the task owns, but elsewhere
    only within a few ulp of the truth (docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel
    (T-0971)").
  - The first shipped attempt applied the accurate Q only on [a + 1, 3a]. The 3a cut was placed to keep
    `CHIDIST(42,7)` = Q(3.5, 21) (gen-m9/fec39efaba9284c3 A249) matched, where Excel is 1 ulp above the correctly
    rounded Q and the double kernel has Excel's bits. The fidelity review found that a keep-matched cut, which
    decisions 0056 and 0082 forbid.
  - The lower cut at a + 1 is evidenced, not fitted: below it Excel is the series' 1 - P bit for bit
    (`CHIDIST(5,5)` = Q(2.5, 2.5), gen-m9/4250f880ab640c49 A331, and `CHIDIST(7,7)` = Q(3.5, 3.5), gen-m9/
    899f9555e67da2d6 A72, which the engine's 1 - P matches and the accurate Q misses by 3 and 1 ulp).
- **Decision:**
  - CHISQ.DIST.RT (with CHIDIST and CHISQ.TEST) takes the accurate half-integer Q for every x >= a + 1 up to the
    kernel's MAX_X (650), with no fitted upper cut.
  - The gated cells this costs are accepted under decision 0083's ulp allowance: each stays numeric, within 4 ulp of
    Excel and within 15 significant digits, so none becomes visibly off. They are listed under Consequences.
  - T-3285's PR may run `rustsheet-eval update-baseline --allow-regressions` for exactly these cells.
- **Consequences:**
  - The cells lost or moved farther (main against the PR head, per cell): see the list below.
  - quality/fitted.json loses half_integer.rs's one FITTED tag (the 3a cut is gone).
  - Excel's own form for Q at these shapes (T-3269's far tail, and its follow-up) may later move these cells back to
    Excel's bits; until then Excel's value stays the expected one and the cells are lines of eval/baseline.
