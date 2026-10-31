---
id: T-3396
title: "Probe the df 1 Student-t edges T-2858 left unprobed: T.INV(0.5,1), TINV at 1-2^-53 and 1-2^-52, CONFIDENCE.T overflow"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for three df = 1 Student-t points T-2858 implemented without a probe, recorded in statistics.md, and the
engine matching it.

## Context
Follow-up from T-2858's review (PR #2437, reviewer-excel-fidelity and reviewer-security MINOR findings):
- `T.INV(0.5,1)`: the engine returns 0 (today's value; T-2858's task says "the 0.5 cell is unprobed"). Probe it, and
  the sign of the zero.
- Two-tailed p just below 1 at df 1: Excel gives 0 at `TINV(1,1)` and `TINV(1-1E-17,1)` (gen-m9 0620505280d2ea6e A90,
  7f7ca726c3f68554 A166; the literal reads as 1) and the 1/TAN form at `T.INV.2T(0.999999999999999,1)`
  (probes/t3349-t-inv-2t-near-one A10). Between them, p = 1-2^-53 and 1-2^-52 (computed, e.g. `1-2^-53`) are unprobed:
  the engine gives `1/excel_tan(PI*p/2)` there and 0 only at p = 1 exactly (`t_inv_of` in
  functions/statistical/t_dist.rs).
- `CONFIDENCE.T` with a product past the double range, e.g. `CONFIDENCE.T(1E-10,1E307,2)` and
  `CONFIDENCE.T(1E-15,1E307,3)`: the engine returns `#NUM!` (`confidence_t_of` in functions/statistical/normal.rs,
  `TODO(T-3396)`); the error kind is unverified.
Use the `excel-probe` skill; one new probes case (M9).

## Acceptance criteria
- [ ] A filled probe case covers the points above; statistics.md's T-2858 entry records the results.
- [ ] The engine matches every probed cell (or a follow-up task owns the difference), and `TODO(T-3396)` is gone.
- [ ] `make check` green; reviewers approve.

## Out of scope
- df >= 2.
