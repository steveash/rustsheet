---
id: T-3473
title: "BETA.DIST density at a shape of 5 to 100: 3 m9-beta-dist cells where Excel is within 2.5 ulp of the math and ours is 12 to 38 ulp off (accuracy)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The 3 BETA.DIST density cells below, at shapes (10, 5), (100, 5) and (5, 100), are within half a unit of the 15th
significant digit of Excel's value through a more accurate density (not a fit to Excel's bits), with no density cell
that is within 15 digits today moving off.

## Context
- Gap triage 2026-10-31i (`make eval` on origin/main 0c2fbf706, excel-graph; `scripts/exit_owners.py --all-off` and
  `--milestone M9`, `rate:BETA.DIST`): no open owner, deferred or not. They are the density cells of the unowned
  m9-beta-dist group where Excel is close to correctly rounded; the rest are T-3475's (Phase 1b).
- Signed ulps from mpmath at 60 digits (x^(a-1)(1-x)^(b-1)/B(a, b) with x the double Excel reads), Excel / ours:
  - m9-beta-dist-a10 A32 `BETA.DIST(0.1,10,5,FALSE)`: Excel 6.567561000000001E-6 (-2.5), ours 6.567560999999993E-6 (-11.9)
  - m9-beta-dist-a100 A34 `BETA.DIST(0.5,100,5,FALSE)`: Excel 4.534102298350339E-23 (+2.3), ours 4.53410229835036E-23 (+38.0)
  - m9-beta-dist-a5 A52 `BETA.DIST(0.5,5,100,FALSE)`: the mirror point, same values (+2.3 / +38.0)
- Root cause (suspected): both shapes > 1, so the density is the Loader saddle-point form (statistics.md "Beta density
  for min(a,b) > 1 is Loader's saddle point (T-2464, T-2513)"; `beta_density.rs`). At (5, 100) the point x = 0.5 is far
  in the tail (|E| about 51), where the error tracks ulp(E): a stored double at E's scale, or the bd0 / stirlerr terms
  at n = 103, is where 38 ulp can come from. At (10, 5) the tail is milder. Measure each term against mpmath before
  changing anything.
- R5 note: decision 0188 R5 parks last-digit bit-reproduction of the BETA.DIST density as Phase 1b. These three are not
  that: Excel is within 2.5 ulp of the math and ours is 12 to 38 ulp off, so the fix is accuracy. If a term turns out
  to need Excel's own rounding, that part moves to a `visible` follow-up with `defer: phase-1b` and the R4 guards of
  reviewer-excel-fidelity check 7.
- Code: crates/rustsheet-engine/src/functions/statistical/beta_density.rs, `large_shape_beta.rs`.

Owned cells (all BETA.DIST density):
- probes-numeric/m9-beta-dist-a10.case.json (1): Calc!A32
- probes-numeric/m9-beta-dist-a100.case.json (1): Calc!A34
- probes-numeric/m9-beta-dist-a5.case.json (1): Calc!A52

## Visible-parity stage
Decision 0083 / 0114 (`accuracy`): met when the 3 cells above are within half a unit of the 15th significant digit
of Excel's value, and no BETA.DIST density cell in probes-numeric that is within 15 digits today moves off. Bit-level
criteria still unmet then move to a follow-up with `defer: phase-1b`.

## Acceptance criteria
- [ ] The 3 cells above are within half a unit of the 15th significant digit of Excel's value (`make eval-show` output
      in the Outcome).
- [ ] The count of BETA.DIST density cells within 15 digits of Excel across probes-numeric does not fall (before/after
      counts in the Outcome), and no gated cell goes from within 15 digits to visibly off.
- [ ] A unit test pins the fixed cells at Excel's value, with the case and cell named.
- [ ] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Density cells where Excel is off the math (T-3475, T-3173, T-3174; Phase 1b). The cdf (T-3472, T-3401).
