---
id: T-3246
title: "IRR regressed on calipers formulas_financial E132 after T-3120 (2 ulps off), blocking the calipers baseline ratchet"
milestone: M11
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`=IRR($C$47:$C$51,0.5)` in calipers `roundtrip/formulas_financial` Sheet1!E132 matches Excel's
cached value bit for bit again. `rustsheet-eval update-baseline --suite calipers` then records
without a regression.

## Context
Found by T-3230 on 2026-10-30, when `update-baseline --suite calipers` refused one regression:
- Excel's cached value is 0.15322137877183528; the engine gives 0.15322137877183506
  (`make eval-show CASE=calipers/roundtrip/formulas_financial/golden.xlsx`).
- The cell passed at the last calipers baseline update, 2026-10-29 18:07 (commit date). T-3120
  ("run IRR's secant in d = 1 - v", #2389) merged at 23:45 the same day and is the only IRR change
  since.
- Until this is fixed, the calipers baseline cannot be ratcheted: 2,862 fixed cells (including
  `pv26_002` and `pv26_2_002`, back since T-3230) wait to be recorded. calipers is not a gated
  suite, so `make check` is unaffected.

Code: `crates/rustsheet-engine/src/functions/financial*` (IRR secant). Check whether the guess 0.5
starts the iteration differently from the probes T-3120 fitted.

## Acceptance criteria
- [ ] calipers formulas_financial E132 passes, T-3120's own probe cases still pass, and a unit
      test pins this input.
- [ ] `rustsheet-eval update-baseline --suite calipers` records without `--allow-regressions`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
