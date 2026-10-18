---
id: T-1785
title: "Move the date and time lifts, MUNIT, MROUND and SQRTPI onto the SPEC-0011 driver and delete lift_numbers"
milestone: M9
lane: quality
status: blocked
priority: P1
size: M
tier: opus
depends_on: [T-1770, T-1773, T-1932]
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

## Blocked because
The bench criterion is not met, and meeting it needs the shared driver hot path changed (T-1932), beyond this task.
Everything else is done on branch `agent/quality/T-1785-date-time-lifts-driver`:
- DAY/MONTH/YEAR, HOUR/MINUTE/SECOND, DATE, TIME, DAYS, DAYS360, DATEDIF, WEEKDAY and ISOWEEKNUM are `Lift` entries; MROUND (`ATP_2`) and
  SQRTPI (`ARRAYS_1`) are `LiftArrays` entries; their static-analysis fallback arms are gone. MUNIT and RANDBETWEEN call `lift_values`
  themselves (MUNIT's scalar result is a matrix; RANDBETWEEN keeps its own range rule). INDEX reads its operands with `bcast_get`.
  `lift_numbers`, `lift_number_operands`, `number_at`, `date_part_fn!` and `time_part_fn!` are gone; `apply_number_unary` serves only
  `apply_unary_chain`. Kernel errors carry no message.
- Eval: every case of all 16 suites scores the same as main except the new probe probes/date-math-lift-driver-edges (55 cells,
  excel-graph): main 26, branch 54. The one failing cell, G43, is T-1430's error order. The probe found DATEDIF's unit and DAYS360's
  method lift over arrays and ranges (T-1428's half), the ATP rule for MROUND/SQRTPI over TAKE/DROP, 3-D references (`#REF!` for
  `Lift`, `#VALUE!` for `LiftArrays`) and the legacy `@` (none for SQRTPI/MROUND, K2-K3); recorded in dates.md, arrays.md, numbers.md.
- Bench (`math_lift`, min of 11, alternating processes, noisy 4-core box): DAY per row main 16.4-18.7 ms vs branch 19.4-26.5 ms; callgrind
  `Model::evaluate` +9.4% instructions (2.109G vs 2.308G). HOUR, DATE per row and `SUM(DAY(A1:A1000000))` are 10-25% slower too.
Resume once T-1932 is done: merge main, re-run the bench, then `make check`, the review skill and the PR.
