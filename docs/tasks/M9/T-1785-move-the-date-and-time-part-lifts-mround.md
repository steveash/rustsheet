---
id: T-1785
title: "Move the date and time lifts, MUNIT, MROUND and SQRTPI onto the SPEC-0011 driver and delete lift_numbers"
milestone: M9
lane: quality
status: done
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
- [x] `make eval`: 0 fixes and 0 regressions in every suite (probes/date-argument-modes, ceiling-floor-mround-array-lift, sqrtpi-range-operands, mod-array-lift and round-concat-array-edges keep passing).
- [x] `rg -n "fn lift_numbers|fn lift_number_operands|fn number_at" crates/rustsheet-engine/src` finds nothing, and `apply_number_unary` has no caller outside the unary-operator chain.
- [x] A per-row unlifted `=DAY(A{r})` bench over at least 100k rows with a full recalc is within 5% of main; the numbers are in the Outcome. Relaxed by decision 0077 (owner, 2026-10-19): landed at +9.4% instructions, the target moves to T-1932.
- [x] Kernels on the driver build element errors with an empty message (`String::new()`).
- [x] `make check` green; reviewers approve.

## Out of scope
- WORKDAY and NETWORKDAYS (T-1236). EDATE, EOMONTH, WEEKNUM and YEARFRAC (T-1770).

## Outcome
- **Moved onto the SPEC-0011 driver** as table entries, with their static-analysis fallback arms deleted:
  - `Lift`: DAY, MONTH, YEAR, HOUR, MINUTE, SECOND, DATE, TIME, DAYS, DAYS360 (new `LIFT_2_OF_3`), DATEDIF, WEEKDAY, ISOWEEKNUM. DAYS, DAYS360 and TIME are beyond the Goal's list: `lift_numbers` could only go if they moved too.
  - `LiftArrays`: MROUND (`ATP_2`), SQRTPI (`ARRAYS_1`).
  - MUNIT and RANDBETWEEN call `lift_values` themselves (MUNIT's scalar result is a matrix, so its static shape stays unknown; RANDBETWEEN keeps its own range rule and reads `top` first). INDEX's index lift reads its operands with `bcast_get`.
  - `lift_numbers`, `lift_number_operands`, `number_at`, `date_part_fn!` and `time_part_fn!` are gone; `apply_number_unary` serves only `apply_unary_chain`. Kernel errors carry no message; DATEDIF's unit and DAYS360's method are read without a per-element allocation. `numbers_fn` skips the coercion chain for a number argument.
- **Eval:** every case of all 16 suites scores as on main, apart from the new probe probes/date-math-lift-driver-edges (69 cells, excel-graph): T-1785 passes all but G43, T-1430's error order (main passed 26 of its first 55). Its N1-N13 settle the corners the Excel-fidelity review found changed without evidence: a one-cell range or reference is its value, not a 1x1 array (`ROWS(MUNIT(E1:E1))` is the whole matrix), and an MROUND range beside an array gives per-element `#VALUE!`. It found that DATEDIF's unit and DAYS360's method lift over arrays and ranges (T-1428's half), a legacy DATEDIF intersects its unit range with the formula's row (K1, L2), MROUND and SQRTPI refuse TAKE/DROP results and take no legacy `@` (K2, K3), and a 3-D reference is `#REF!` in a `Lift` argument and `#VALUE!` in a `LiftArrays` one; dates.md, arrays.md and numbers.md record them. Duplication 3861 -> 3831.
- **Bench** (callgrind `Model::evaluate`, main 39385aff vs this branch): DAY per row +9.5%, DATE per row +10.0%, `SUM(DAY(A1:A1000000))` +13.0%, HOUR per row -12.3%, ABS per row 0.0%. Over the 5% target; the owner relaxed it (decision 0077) and T-1932 owns the gap.

