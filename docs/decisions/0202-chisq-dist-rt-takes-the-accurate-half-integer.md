# 0202 — CHISQ.DIST.RT takes the accurate half-integer Q for every x >= a + 1 (owner ruling)

- **Status:** accepted (2026-10-31, T-3285; owner ruling (a) "Allow, with records", session
  https://claude.ai/code/session_01WTJ7rq9tsQyspbybqNLk1a). Applies decision 0083's standing ulp-regression allowance to
  T-3285's PR, with the cells below named. Amended 2026-10-31 by the owner's extended ruling (a) ("Extend both
  rulings", same session): the ruling also covers the two ungated probe cells the PR makes newly visibly off, named under
  Consequences.
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
  - Main (1af9bd6c6) against T-3285's head, per cell over every suite, in ulps of Excel's value. Gated:
    - lost (bit-exact to not): gen-m9/fec39efaba9284c3 Calc!A249 `CHIDIST(42,7)` = Q(3.5, 21), 0 to 1;
    - farther: gen-m9/320d51765af9b6b7 Calc!A249 and 7674c89c824aca43 Calc!A332 `CHIDIST(9,5)` = Q(2.5, 4.5), 2 to 3
      (Excel is -2.6 ulp from the truth); gen-m9/b8ee55934c5b824c Calc!A262 `CHIDIST(37.5,7)` 2 to 3 and Calc!A400
      `CHIDIST(37.5,5)` 1 to 2;
    - none newly visibly off; every one numeric, within 4 ulp and within 15 digits of Excel (decision 0083 conditions
      1 and 2). Fixed 4 and visibly fixed 2 (ironcalc-calc CHISQ.xlsx G3 and G4), more than regressed (condition 3).
  - Ungated (probes-numeric) cells the PR loses or makes newly visibly off are listed in T-3285's Outcome; probes-numeric
    has no baseline and is Phase 1b evidence (decision 0188 R1(c)). Every changed value is within Gate A's envelope.
    The extended ruling (a) names and accepts the two newly visibly off, in ulps of Excel's value, main to HEAD:
    - probes-numeric/t3284-gamma-deep-upper-tail Calc!A246 `CHISQ.DIST.RT(56,7)` = Q(3.5, 28), 2 to 3;
    - probes-numeric/t3284-gamma-deep-upper-tail Calc!A260 `CHISQ.DIST.RT(120,15)` = Q(7.5, 60), 11 to 13.

    Both are past x = 30, Excel's deep upper tail (T-3269's region, not T-3461's), and both stay within Gate A's
    envelope; T-3285's Outcome lists them.
  - T-3461 (deferred to Phase 1b, R5) names the regressed cells and looks for Excel's own form on [a + 1, 30]
    (condition 4); T-3269 covers x > 30.
  - eval/baseline/gen-m9.txt gains the line for `CHIDIST(42,7)` and loses the fixed cells' lines;
    eval/baseline/ironcalc-calc.txt loses CHISQ.xlsx G3; `update-baseline --allow-regressions` writes both.
  - Against main, quality/fitted.json is unchanged: the kernel carries no fitted piece (an earlier commit on this branch
    added a 3a cut and its tag, and both are gone).
  - New coverage, not a loss: on this PR's own new probe, probes-numeric/t3285-chisq-dist-rt-half-integer-sweep,
    A51 and A101 (both past x = 25, T-3269's region) are visibly off on this head as they are on main's engine; no
    existing Excel evidence is lost (T-3461 and T-3469 own the region).
  - Decision 0188 R4: with no fitted piece, R4.1's hold-out does not apply; R4.2's fresh sweep
    (probes-numeric/t3285-chisq-dist-rt-half-integer-sweep, docs/research/t3285) is in T-3285's Outcome.
