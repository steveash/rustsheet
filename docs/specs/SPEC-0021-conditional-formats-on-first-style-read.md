# SPEC-0021 — Conditional formats evaluated on the first style read

- **Status:** draft
- **Milestone / lane:** M11 / perf
- **Tasks:** T-0475 (this spec), T-0468 (implementation)

## Problem
`Model::evaluate_conditional_formatting` (conditional_formatting.rs) runs at load
(`Model::from_workbook*`, new_empty.rs), at the end of every `evaluate()` (evaluation.rs), and in
`set_theme`. It evaluates every rule over every cell of its area into `cf_cache`, which only
`get_extended_style_for_cell` and `UserModel::get_extended_cell_style` read. Value evaluation never
reads it, and the eval harness, the Python bindings and the `rustsheet` crate never read it at all.

Measured, release, corpus-bench `41978/1_41978_input.xlsx` (sheet A1:M185):
- load takes 184 s, then `evaluate()` takes 191 s, all of it inside `evaluate_conditional_formatting`;
- the cause is expression rules over near-million-row areas (`G186:G1047468` with
  `IF(J195:J619,"open")`, `G1047889:G1048576` with `IF(J1:J1047898,"open")`);
- `apply_cf_formula` evaluates the formula once per cell, about 1.6e9 array elements per run, and
  discards every result: an array counts as no match.

The file is a `!Timeout` line in eval/baseline/corpus-bench.txt (T-0430).

## Goals / non-goals
Goals:
- Loading and `evaluate()` do no conditional-format work.
- The first style read after a change evaluates the conditional formats once, then reads the cache
  as today.
- The style a read returns is the one today's eager evaluation would have stored, given the same
  workbook state.

Non-goals:
- Making one conditional-format evaluation over huge areas fast.
- What Excel does with a rule formula that returns an array (T-2716).

## Excel behavior to match
Unchanged: every rule is evaluated by today's code. The only question is when it is evaluated.

## Design
- **Stale flag.** `Model` gains `cf_stale: bool`. These set it instead of evaluating: the end of
  `evaluate()`, load, `set_theme`, and every edit of the conditional formats or of the dxf styles
  they reference.
- **Mutable style reads.** These public methods change from `&self` to `&mut self`:
  - `Model::get_extended_style_for_cell(&mut self, ..)`;
  - `UserModel::get_extended_cell_style(&mut self, ..)`.

  When the flag is set they call `evaluate_conditional_formatting()` and clear it, then read
  `cf_cache` as today. They need `&mut` because rule formulas are evaluated through
  `evaluate_outside_cell`. Their callers are the engine's and rustsheet-xlsx's tests, plus
  `UserModel`. No binding or `rustsheet` API exposes them.
- **Public refresh.** `Model::evaluate_conditional_formatting` stays public for callers that want
  the work done eagerly.

### When the evaluation happens, and what it can see
Today the cache is a snapshot taken at the end of `evaluate()`, or at load. The lazy evaluation
runs at the first style read instead. The two agree whenever nothing changes in between, which
covers the eval harness and `UserModel`, since `UserModel` evaluates after every edit
(`evaluate_if_not_paused`). Two differences remain, and both are accepted:
- **Edits made through `Model`, or through a paused `UserModel`, without `evaluate()`.** If cells are changed between `evaluate()`
  and the first style read, the conditional formats see the changed inputs. Today they would have
  seen the snapshot. Formula cells are not re-evaluated by this; they keep their last values, as
  today.
- **Values read between load and the first `evaluate()`.** At load, today's evaluation evaluates
  the formula cells the rules read (`evaluate_cell` outside a pass) and stores their values. So a
  value read after load could be a recomputed one rather than the file's cached one. With the
  change, values read after load are the file's cached values until `evaluate()` or a style read.
  That matches Excel, which opens a workbook showing its cached values.

## Alternatives considered
- **A static "this rule can never match" check** (a rule formula that is always an array). It
  fixes 41978 only, and skipping an evaluation still changes the load-time side effect above.
- **Interior mutability (`RefCell`) for the cache.** Rule evaluation needs `&mut Model`, not just
  a mutable cache.
- **A cap on conditional-format work** (directive 5). It would change styles, needs a DEVIATION
  entry, and still charges every load.

## Test plan
- The conditional-formatting tests pass unchanged, with only the receivers changed (engine
  `test/conditional_formatting`, `rustsheet-xlsx/tests/test_conditional_formatting.rs`).
- New tests:
  - a load or an `evaluate()` followed by no style read evaluates no rule (a test-only counter);
  - the first read after `evaluate()`, `set_theme` or a conditional-format edit sees the new state;
  - a second read does not re-evaluate.
- `rustsheet-eval check`: 0 regressions in every gated suite.
- corpus-bench 41978 runs under 60 s through the harness, and its `!Timeout` line leaves the
  baseline.

## Rollout
1. **T-0468 (M).** The flag, the receivers, the call sites and the tests above, measured on 41978.
