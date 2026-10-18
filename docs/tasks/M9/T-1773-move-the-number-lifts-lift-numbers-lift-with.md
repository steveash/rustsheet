---
id: T-1773
title: "Move the math number lifts (single_number_fn!, ROUND family, MOD, CEILING/FLOOR, lift_with_defaults) onto the SPEC-0011 driver"
milestone: M9
lane: quality
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The math number lifts are table entries and kernels on the SPEC-0011 driver, with no result change. That covers the 34 `single_number_fn!` functions, the ROUND family, MOD and CEILING/FLOOR. `single_number_fn!`'s body, `lift_with_defaults` and the math callers of `lift_numbers` are gone (directive 3).

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "Today's lifting code" and "Performance".
- Split by T-1767's review from the date, time, MROUND and SQRTPI part (T-1785), because these are about 60 functions on a hot path.
- Files: functions/macros.rs, functions/math_and_trigonometry/mathematical.rs, cast.rs.
- `lift_numbers` and `lift_number_operands` are deleted in T-1785, once their last caller moves.
- ABS is the subject of the existing `test_nested_formula_stack` test (src/test/test_nested_formula_stack.rs), so that test must keep passing after ABS moves.
- Each function's static-analysis arms move into the table. Every entry is `KernelWork::Constant` unless its loop depends on argument values (SPEC-0011 security rule).

## Acceptance criteria
- [ ] `make eval`: 0 fixes and 0 regressions in every suite.
- [ ] `rg -n "macro_rules! single_number_fn|fn lift_with_defaults" crates/rustsheet-engine/src` finds nothing. `scripts/check_duplication.py` does not rise, and any drop is recorded with `--update`.
- [ ] A criterion bench in crates/rustsheet/benches/ covers:
  - per-row unlifted cells over at least 100k rows with a full recalc: `=ROUND(A{r},2)`, `=ABS(A{r})`, `=MOD(A{r},7)`;
  - lifted `=SUM(ROUND(A1:A1000000,2))` and `=SUM(ABS(A1:A1000000))`;
  - a lifted error fan-out, `=SUM(IFERROR(ROUND(A1:A1000000&"x",2),0))`.

  Each runs on main and on the branch in release and is within 5%; the numbers are in the Outcome.
- [ ] Kernels on the driver build element errors with an empty message (`String::new()`), so an error element allocates nothing.
- [ ] `test_nested_formula_stack` passes unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The date and time parts, DATE, DATEDIF, MUNIT, MROUND and SQRTPI (T-1785). The operators' `broadcast_arrays` and `apply_unary_chain`.
