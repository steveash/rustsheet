---
id: T-1932
title: "Cut lift_call's per-call overhead so a one-number kernel costs what apply_number_unary did"
milestone: M9
lane: perf
status: todo
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
Code: crates/rustsheet-engine/src/functions/lift/call.rs, driver.rs, mod.rs (`kernel_number`), math_and_trigonometry/mathematical.rs (`numbers_fn`, `number_fn`).

## Acceptance criteria
- [ ] On T-1785's code, `math_lift` "DAY, per row", "DATE, per row" and "DAY, lifted" are each within 5% of main before T-1785 (39385aff: callgrind `Model::evaluate` 2.107G, 3.135G and 16.77G; decision 0077 has the table), and "ABS, per row" and "HOUR, per row" do not get slower.
- [ ] The stack-depth and recursion tests pass unchanged; no new `unsafe`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
