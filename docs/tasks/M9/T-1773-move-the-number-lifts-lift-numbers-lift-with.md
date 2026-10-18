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

## Progress (2026-10-18, opus run; branch pushed, resumable)
Done on agent/quality/T-1773-math-lifts-driver:
- The 32 `single_number_fn!` functions are `LIFT_1` entries with one kernel helper,
  `Model::number_fn` (mathematical.rs); `functions/macros.rs` is deleted. The ROUND family, MOD,
  CEILING/FLOOR and the four `lift_with_defaults` callers are `LIFT_2`, `LIFT_1_OF_2` and
  `LIFT_1_OF_3` entries over `Model::numbers_fn` (defaults for the omitted trailing arguments);
  `lift_with_defaults` is gone. CEILING/FLOOR moved to math_and_trigonometry/ceiling_floor.rs (file
  size). Their static-analysis fallback arms are removed (the table derives the same signatures and
  shapes). Kernel errors carry no message (`kernel_number`, `number_result` in functions/lift/mod.rs).
- `make eval-check` in the worktree: 0 fixes, 0 regressions (fixtures, gen-*, ironcalc-calc, probes,
  smoke; poi not fetched there). Engine unit tests pass, `test_nested_formula_stack` included.
  Duplication 4812 -> 4792 (run `scripts/check_duplication.py --update` after the final main merge).
- Bench `crates/rustsheet/benches/math_lift.rs` (one case per process, min of 11 runs, release):

  | case | main | branch |
  |---|---|---|
  | ROUND, per row (100k) | 132-137 ms | 150 ms |
  | ABS, per row (100k) | 17.6 ms | 28.3 ms |
  | MOD, per row (100k) | 21.7-22.9 ms | 32.8 ms |
  | SUM(ROUND(A1:A1000000,2)) | 1.58 s | 1.26 s |
  | SUM(ABS(A1:A1000000)) | 324 ms | 258 ms |
  | SUM(IFERROR(ROUND(A1:A1000000&"x",2),0)) | 2.31 s | 2.44 s |

Blocking the 5% criterion: the driver's unlifted path costs about 100 ns more per call than
`get_number_or_array` did. About 60% of it is `ArgBuf`: `ArgBuf::filled` builds, moves (through
`read_lift_args`' `Result<ReadArgs, _>`) and drops all `INLINE_ARGS` = 8 `CalcResult` slots for a
one-argument call (with `INLINE_ARGS` = 2 as an experiment, ABS per row is 22.5 ms). Next: make the
read phase build only `len` slots (e.g. a `SmallVec`-like length-tracked buffer, or a fast path for
one- and two-argument calls in `lift_call_with` that reads into a local array), then profile the
rest (the `dyn` kernel call, `arg_kinds()` twice per call); the error fan-out case (+5%) likely
shares the cause. Then rerun the bench, fill the Outcome, review and PR.
