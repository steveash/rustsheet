# 0166 — Allow the YIELD and ODDFYIELD ulp regressions of T-2974's unrounded-term start

- **Status:** accepted (2026-10-29, T-2974 attempt 4)
- **Context:**
  - T-2974 attempt 4 (financial.md "YIELD stops short of PRICE's root", T-2974 attempt 4) found
    the arrangement behind attempt 3's tie anomaly: Excel's start reads `pr T` with `T` as the
    exact sum `whole + fl(days/year)`, not that sum rounded, and rounds `pr T + 100 T` once.
    In the fit lab (`^`-priced PRICE, 1,115 YIELD probe cells) it makes 960 bit-exact against
    790 for attempt 3's FMA form and 709 for the engine's rounded-once start (decision 0162).
    It is the only candidate found that makes yield-solver-sweep C7, yield-start-rate-term B39
    and B49 and yield-newton-domain P!B2 exact.
  - In the engine it fixes 515 known failures (probes 494, ironcalc-calc 12, gen-m9 9),
    ODDFYIELD among them since it shares the start. In-scope visibly-off cells go from 3,166 to
    3,141.
  - It moves 58 cells off the bit: 48 YIELD and 10 ODDFYIELD (probes 57, gen-m9 1). 51 are 1 ulp
    off, 5 are 2, 1 is 3. On the YIELD cells the lab model gives the same distances, so these are
    misses of the model, not of the port.
  - One becomes visibly off: probes/yield-one-step-start S!A37 (32 ulps). It is a one-step cell
    whose new start is one ulp off Excel's and crosses a step of PRICE's `^` base. Decision
    0083's standing allowance does not cover it.
- **Options considered:**
  - *Keep the rounded-once start.* 515 fewer fixed cells, and C7, B39 and P!B2 stay visibly off.
  - *Attempt 3's FMA form.* Fixes 466 and regresses 196, one of them visibly (yield-solver-sweep
    E12, 64 ulps).
  - *Fit further first.* The remaining misses (155 of 1,115 in the lab) have no named candidate
    yet; T-2974's time box asks for the best candidate to land.
- **Decision:** run `rustsheet-eval update-baseline --allow-regressions` once in the T-2974
  attempt 4 PR, for exactly these 58 cells. The follow-up T-3152 owns them, A37 included.
- **Consequences:** the baselines gain exactly these 58 lines and lose the fixed ones. Reviewers
  check that the added lines are all YIELD or ODDFYIELD cells. The unit test
  `fn_yield_start_term_whole_years_then_leftover_days` pins its A1 (probes/yield-start-term-basis
  B5) within one ulp until T-3152. The visible-parity lock is not lowered here; the steward
  lowers it (decision 0093).
