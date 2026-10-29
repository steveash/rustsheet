---
id: T-3058
title: "CHISQ.TEST and CHITEST take the p-value from the direct upper tail, not 1 - P"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
campaign: accuracy
---

## Goal
CHISQ.TEST and CHITEST return Excel's p-value to the last digit wherever it is small: the right tail
Q(df/2, chi2/2) computed directly, as CHISQ.DIST.RT does, instead of 1 - P.

## Context
T-3021 triage. `fn_chisq_test` (crates/rustsheet-engine/src/functions/statistical/chisq.rs) takes
`chi_squared_sf` only at one degree of freedom and `1.0 - gamma_cdf(&dist, chi2)` otherwise. That
cancels digits as p gets small. statistics.md "Incomplete gamma: accurate kernel (T-0971)" already says
Excel's upper tail "is not 1 - P", and CHISQ.TEST is in that rule's list.
- probes-numeric/m9-legacy-chitest Calc!A23 `CHITEST(F1:F3,G1:G3)` (F = {5,0,"x"}, G = {1,2,3}: chi2 = 18,
  df 2): Excel 0.00012340980408667956, which is `EXP(-9)` (the closed form at df 2) to the last bit;
  ours 0.00012340980408664937 (13th digit, visibly off). This is CHITEST's one unowned visibly-off cell (T-3021).
- Same case A9/A10 (chi2 = 5.55, df 2): Excel 0.06234947668967343 = `EXP(-2.775)`, ours ...339; A3/A4 1 ulp.
Measured by T-3021 with `let mut p = chi_squared_sf(&dist, chi2);` for every df (the df-1 branch is kept,
since `chi_squared_sf` takes it itself):
- probes-numeric/m9-legacy-chitest goes from 26/31 to 31/31;
- probes-numeric/t1049-chisqtest goes from 21/29 to 28/29. A11 `CHISQ.TEST(C1:C4,B1:B4)` stays 1 ulp off:
  Excel 0.7710426682428904, ours ...905;
- `rustsheet-eval check`: 0 regressions and 0 fixed in every gated suite (main 5b8c2ae7);
- ironcalc-calc/STATISTICAL/CHISQ_TEST.xlsx E1 still passes.
The `p > 1` / `p < 0` clamps that follow stay; a direct Q never exceeds 1.

## Acceptance criteria
- [ ] `fn_chisq_test` computes p as the direct right tail for every df, with a code comment citing
      m9-legacy-chitest A23 and statistics.md's T-0971 rule.
- [ ] probes-numeric/m9-legacy-chitest passes 31/31, and t1049-chisqtest has no failing cell except A11
      (1 ulp, or fixed).
- [ ] A unit test pins `CHITEST` at chi2 = 18, df 2 to `exp(-9)`.
- [ ] statistics.md's T-0971 entry says CHISQ.TEST now takes the direct tail (T-3058). No gated baseline gains
      an F line; `make check` green; reviewers approve.

## Out of scope
- The incomplete gamma kernel itself (T-0971 done; T-2367 its ulp residue), and the degrees of freedom of two
  different shapes (T-2650).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. It owns m9-legacy-chitest Calc!A23, and fixing it lifts
CHITEST to the Phase 1 rate (T-3021).

## Outcome
`fn_chisq_test` takes `chi_squared_sf` (the direct right tail) at every df. New unit test pins `CHITEST` at chi2 = 18, df 2 to
`exp(-9)` bit for bit. statistics.md's T-0971 rule notes it. Eval: `rustsheet-eval check` 0 regressions, 0 fixed, no baseline
change (probes-numeric is not gated); m9-legacy-chitest A23 now matches Excel.
