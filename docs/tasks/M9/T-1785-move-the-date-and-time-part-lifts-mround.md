---
id: T-1785
title: "Move the date and time lifts, MUNIT, MROUND and SQRTPI onto the SPEC-0011 driver and delete lift_numbers"
milestone: M9
lane: quality
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-1770, T-1773]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The date and time lifts and the hand-written Analysis ToolPak range checks are table entries and kernels on the SPEC-0011 driver, with no result change. This covers DAY/MONTH/YEAR, HOUR/MINUTE/SECOND, DATE, DATEDIF, WEEKDAY, ISOWEEKNUM, MUNIT, MROUND and SQRTPI. `lift_numbers`, `lift_number_operands`, `number_at`, `date_part_fn!`'s and `time_part_fn!`'s bodies and the function callers of `apply_number_unary` are gone.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Today's lifting code". The second half of the split of T-1773.
- Files: functions/date_and_time.rs, cast.rs, mathematical.rs (MROUND, SQRTPI), matrix.rs (MUNIT).
- MROUND and SQRTPI become `LiftArrays` (arrays.md "CEILING/FLOOR family and MROUND lift over arrays", numbers.md SQRTPI). MROUND's left-to-right argument judging must be kept by its kernel.
- DATEDIF's date-text reading in the workbook's date system (T-1235, T-1330) moves into its kernel.
- `NumberOrArray` and `apply_number_unary` stay for the operators only.

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite (probes/date-argument-modes, ceiling-floor-mround-array-lift, sqrtpi-range-operands, mod-array-lift and round-concat-array-edges keep passing).
- [ ] `rg -n "fn lift_numbers|fn lift_number_operands|fn number_at" crates/rustsheet-engine/src` finds nothing, and `apply_number_unary` has no caller outside the unary-operator chain.
- [ ] A per-row unlifted `=DAY(A{r})` bench over at least 100k rows with a full recalc is within 5% of main; the numbers are in the Outcome.
- [ ] Kernels on the driver build element errors with an empty message (`String::new()`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- WORKDAY and NETWORKDAYS (T-1236). EDATE, EOMONTH, WEEKNUM and YEARFRAC (T-1770).
