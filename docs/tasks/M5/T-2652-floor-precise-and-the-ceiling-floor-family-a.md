---
id: T-2652
title: "FLOOR.PRECISE and the CEILING/FLOOR family: a 16th-digit tie in the ratio's 15-digit cut goes away from zero"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
FLOOR.PRECISE (and every CEILING/FLOOR family member that shares `snap_ratio`, if the probe agrees) matches Excel when the
ratio `value / significance` has an exact tie at its 16th significant digit: the 15-digit cut rounds the tie away from zero,
as INT's cut does, not to even.

## Context
- gen-m5 round-3 triage (T-2648, excel-graph oracle, 2026-10-25), 1 cell in eval/baseline/gen-m5.txt, visibly off:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/048f6b1a0fb9544b Calc!A257 | `=FLOOR.PRECISE(AVERAGE(Data!A10:A12,(100-1000000000000000.0)))` (A10:A12 = "abc", 100, 0.5) | -333333333333267.0 | -333333333333266.0 |

- The AVERAGE is -333333333333266.5 exactly (sum -999999999999799.5 over 3 numbers, every step exact), so FLOOR.PRECISE is
  the innermost divergence. `snap_ratio` (crates/rustsheet-engine/src/functions/math_and_trigonometry/ceiling_floor.rs)
  cuts the ratio with `to_excel_precision`, whose `{:e}` formatting breaks a tie to even (266), then floors. Excel's
  -333333333333267 fits a cut that breaks the tie away from zero (267) before the floor, like INT (numbers.md "INT cuts to
  15 significant digits first (T-2171)", T-2215: `INT(-999999999999999.5)` = -1E+15) and TRUNC.
- Reuse the cut INT and TRUNC use (`round_decimal`, mathematical.rs); do not add another. Note INT's "no cut at 1E+15 and
  above" boundary too: the probe should say whether the family has it.
- Literals with 16 significant digits are cut on entry, so a probe must build the tie from exact arithmetic (as the failing
  cell does: `(100.5+(100-1E+15))/3`, or `-(2^49+0.5)`-style sums), not type it.
- Recorded in docs/excel-semantics/numbers.md "FLOOR.PRECISE: a 16th-digit tie in the ratio cut goes away from zero".

## Acceptance criteria
- [ ] An excel-graph probe (`excel-probe` skill) covers 16th-digit ties, positive and negative, for FLOOR, FLOOR.MATH,
      FLOOR.PRECISE, CEILING, CEILING.MATH, CEILING.PRECISE and ISO.CEILING, plus ratios at and above 1E+15.
- [ ] The engine matches the probe; the gen-m5 line above disappears (`make eval-baseline`), with no new known failure.
- [ ] numbers.md entry updated with the probe evidence and `Status: VERIFIED`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MROUND (its own quotient rule, `mround_value`).
