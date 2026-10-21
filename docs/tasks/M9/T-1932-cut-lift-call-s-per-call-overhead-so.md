---
id: T-1932
title: "Cut lift_call's per-call overhead so a one-number kernel costs what apply_number_unary did"
milestone: M9
lane: perf
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
An unlifted per-row call of a one-number function on the SPEC-0011 driver (`=DAY(A{r})`, `=ABS(A{r})`) costs within 5% of what `apply_number_unary` cost before T-1785, so T-1785's bench criterion holds and every driver function gets cheaper per cell.

## Context
Found by T-1785 (branch `agent/quality/T-1785-date-time-lifts-driver`). Moving DAY onto the driver adds about 165 instructions per call
over main's `apply_number_unary` path: callgrind of `cargo bench -p rustsheet --bench math_lift -- "DAY, per row"` (100k rows, 11 recalcs)
puts `Model::evaluate` at 2.109G instructions on main and 2.308G on the branch (+9.4%; wall-clock min 16.4 ms vs about 19.5-23 ms), after
the branch's `numbers_fn` number fast path already took 55 instructions per call off. There is no single hotspot: the cost is spread over
the stacked frames `numbers_fn` closure -> `lift_call_costed` (about 34 instructions self per call) -> `lift_call_with` (about 68) ->
`read_lift_arg_at`/`read_lift_arg_with` (about 52, plus a `CalcResult` drop) -> the `dyn` kernel. ABS shows the same overhead (it has been on
the driver since T-1773, whose +4.5% was measured when a per-row ABS cost 42 ms; it costs about 18 ms now, so the fixed overhead is a larger share).
`lift_call_costed` is already inlined by LTO (an `#[inline]` changed nothing). Mind T-1110: `lift_call_with` is `#[inline(never)]` so its
frame stays fixed under recursion; a fix must not grow the evaluation stack frame (measure the recursion depth tests).
The lifted row (+13%) is larger than the per-call cost explains: profile the per-element path too (the `dyn` kernel dispatch, and the `CalcResult` built and dropped for each element in `lift_in_place`), not only the unlifted call stack (T-1785 performance review).
Code: crates/rustsheet-engine/src/functions/lift/call.rs, driver.rs, mod.rs (`kernel_number`), math_and_trigonometry/mathematical.rs (`numbers_fn`, `number_fn`).

## Acceptance criteria
- [x] On T-1785's code, `math_lift` "DAY, per row", "DATE, per row" and "DAY, lifted" are each within 5% of main before T-1785 (39385aff: callgrind `Model::evaluate` 2.107G, 3.135G and 16.77G; decision 0077 has the table), and "ABS, per row" and "HOUR, per row" do not get slower.
- [x] The stack-depth and recursion tests pass unchanged; no new `unsafe`.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- **Driver shortcut for number kernels.** `Model::lift_call_numbers` (call.rs) is `lift_call` with a
  `NumbersKernel` shortcut (`dyn Fn(&[f64]) -> Result<f64, Error>`, driver.rs): arguments read as
  numbers stay `f64`s, and when all are, the shortcut runs with no `CalcResult` slice, `dyn` kernel
  call or drop. A one-`Lift`-argument call evaluates its argument in `lift_call_numbers`' own frame
  (fewer frames on the recursion path than `lift_call_with` -> `read_lift_arg_with`). At the first
  non-number the call goes on in `lift_numbers_after` through `read_lift_args_after` on the values
  already read (none is read twice; `read_lift_arg_with` is split into `evaluate_lift_arg` and
  `lift_arg_from_value` for that). In `lift_in_place`, a number element whose scalars are all
  numbers runs the shortcut and writes the `ArrayNode` directly, for entries whose elements are
  charged nothing (`element_is_value`). `numbers_fn` passes its `op` as the shortcut; `number_fn`
  (ABS, SQRT, ...) is now `numbers_fn::<1>` with the NaN check, so every math and date function on
  `numbers_fn` gets cheaper.
- **Bench** (callgrind, `Model::evaluate` as its self cost plus its direct callees: on main the
  inclusive figure is inflated by callgrind's cycle accounting, e.g. ABS per row reads 2.323G
  inclusive against 2.029G summed; same machine and toolchain, 11 recalcs):

  | `math_lift` case | 39385aff (before T-1785) | main 97fce862 | this branch | vs 39385aff |
  |---|---|---|---|---|
  | DAY, per row | 1.996G | 2.224G | 2.090G | +4.7% |
  | DATE, per row | 3.106G | 2.801G | 2.468G | -20.5% |
  | DAY, lifted | 16.254G | 19.083G | 17.029G | +4.8% |
  | ABS, per row | 2.017G | 2.029G | 1.808G (vs main -10.9%) | -10.4% |
  | HOUR, per row | 2.636G | 2.291G | 2.153G (vs main -6.0%) | -18.3% |

  The ABS and DAY rows were re-measured after `number_fn` moved onto `numbers_fn` (see the PR).
- **Stack:** the nesting tests (`test_nested_formula_stack`, `test_nesting_depth`) pass unchanged; no
  `unsafe` (the number is kept from its drop with `ManuallyDrop`, which owns nothing for a number).
- **Eval:** 0 fixes, 0 regressions in every suite. New unit tests in `lift/tests/mod.rs` cover the
  shortcut and the hand-on to the kernel (text first or second, a count error, mixed lifted elements,
  a non-number scalar and a second grid).

