---
id: T-1773
title: "Move the number lifts (lift_numbers, lift_with_defaults, apply_number_unary, single_number_fn!, date and time parts, MROUND, SQRTPI) onto the SPEC-0011 driver"
milestone: M9
lane: quality
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The number lifts are table entries and kernels on the SPEC-0011 driver, with no result change. The old helpers are deleted: `lift_numbers`, `lift_number_operands`, `number_at`, `lift_with_defaults`, the `single_number_fn!`, `date_part_fn!` and `time_part_fn!` bodies, the function callers of `apply_number_unary`, and the hand-written MROUND and SQRTPI range checks (directive 3).

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Migration of the existing helpers".
- Consumers: the ROUND family, MOD, CEILING/FLOOR, MUNIT, DATE, DATEDIF, DAY/MONTH/YEAR, HOUR/MINUTE/SECOND, WEEKDAY, ISOWEEKNUM, the 34 `single_number_fn!` functions, and MROUND and SQRTPI (`LiftArrays`).
- Files: cast.rs, functions/macros.rs, functions/math_and_trigonometry/mathematical.rs, functions/date_and_time.rs and matrix.rs.
- `NumberOrArray` and `apply_number_unary` stay for the operators (`handle_arithmetic`, `apply_unary_chain`).
- Each function's static-analysis lines move into the table.
- The whole-column speed of T-1078 must hold, through the driver's in-place single-grid path.

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite.
- [ ] `rg -n "fn lift_numbers|fn lift_number_operands|fn lift_with_defaults|macro_rules! single_number_fn" crates/rustsheet-engine/src` finds nothing. `scripts/check_duplication.py` does not rise, and any drop is recorded with `--update`.
- [ ] A criterion bench covers `=SUM(ROUND(A1:A1000000,2))`, `=SUM(ABS(A1:A1000000))` and `=SUM(PMT(A1:A100000,10,100))`, run on main and on the branch in release. The first two are within 5%, and the numbers are in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The operators' `broadcast_arrays`. The IS* and text lifts (T-1774, T-1775).
