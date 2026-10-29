# 0162 — Allow the YIELD and ODDFYIELD ulp regressions of T-2974's correctly rounded start

- **Status:** accepted (2026-10-29, T-2974 attempt 2)
- **Context:**
  - T-2974 attempt 2 (financial.md "YIELD stops short of PRICE's root", T-2974 attempt 2) used the
    YIELD probe cells that take one Newton step from the start. On those cells, Excel's value is one
    analytic step from its start. One ulp of start moves that step's PRICE by a price ulp (about 25
    yield ulps), so these cells show the start's last bit. Our formula's start rounds each
    operation in doubles, and on the off cells it is one ulp away from Excel's start
    (yield-solver-sweep C2, C7, C8).
  - The same start, evaluated exactly with an exact term and rounded once, makes 503 of the lab's
    761 YIELD probe cells bit-exact, against 441 for the formula in doubles. x87 64-bit
    intermediates score 498 (the exact value over the double term), and 64 mixes of rounded and exact intermediates score no
    better. In the engine (double-double, `yield_start`), it fixes 245 YIELD and ODDFYIELD known failures in probes,
    9 in ironcalc-calc and 2 in gen-m9. ODDFYIELD shares the start and gains with YIELD: in
    oddfyield-solver-sweep alone, 112 fixed and 8 regressed. In-scope visibly-off cells go from
    2,885 (the lock) to 2,862.
  - It moves 148 cells off the bit: 77 YIELD and 71 ODDFYIELD (probes 139, ironcalc-calc 7,
    gen-m9 2). 139 of them stay within 15 digits. Nine become visibly off:
    - ironcalc-calc PRICE_YIELD!I31, ODDFPRICE_ODDFYIELD!K23 and K24;
    - oddfyield-solver-sweep S!C56 and C88;
    - yield-start-one-input S!A87 and B72;
    - yield-start-rate-term S!B30;
    - yield-start-term-basis S!C47.
  - Decision 0083's standing allowance does not cover the nine visibly-off cells.
- **Options considered:**
  - *Keep the start in doubles.* That keeps 62 fewer bit-exact lab cells and 23 more visibly-off
    cells.
  - *Fit an arrangement that is exact on every cell.* Attempt 1 scored about 20 arrangements and
    attempt 2 scored 64 rounding mixes; none beats the single rounding. C7 and B39 need a start one
    ulp above the correctly rounded one, and no arrangement or perturbed term reaches all of
    those cells (financial.md).
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-2974
  attempt 2 PR, for the probes, ironcalc-calc and gen-m9 suites and exactly these 148 cells. The
  task T-2974 keeps the remaining cells, and the follow-up T-3152 owns the 148.
- **Consequences:** the baselines gain exactly these 148 lines and lose the fixed ones. Reviewers
  check that the added lines are all YIELD or ODDFYIELD cells. The visible-parity lock is not
  lowered here; the steward lowers it (decision 0093).
