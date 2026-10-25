# SPEC-0021 — Conditional formats evaluated on the first style read

- **Status:** draft
- **Milestone / lane:** M11 / perf
- **Tasks:** T-0475 (this spec), T-0468 (implementation), T-0476 (the work bound)

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
- **Stale flag.** `Model` gains `cf_stale: bool`, initialised in the one struct literal
  (new_empty.rs). It is set at exactly the four places that call
  `evaluate_conditional_formatting()` today, which no longer call it:
  - load, in `Model::from_workbook*` (model.rs) and in new_empty.rs, which covers every
    rustsheet-xlsx import;
  - the end of `evaluate()` (evaluation.rs);
  - `set_theme` (model.rs).

  A conditional-format or dxf edit made through `Model` therefore leaves the cache as stale as it
  does today, until the next `evaluate()`. `UserModel`'s edits evaluate (`evaluate_if_not_paused`),
  and so do its undo and redo, so they set the flag. No setter site is added, and none can be missed.
- **Mutable style reads.** These public methods change from `&self` to `&mut self`:
  - `Model::get_extended_style_for_cell(&mut self, ..)`;
  - `UserModel::get_extended_cell_style(&mut self, ..)`;
  - the Python binding's `UserModel.get_extended_cell_style` (bindings/python/src/user_model.rs),
    which becomes a `&mut self` pyo3 method.

  When the flag is set they call `evaluate_conditional_formatting()`, then clear it. That function
  returns nothing and cannot fail, so the cache is never left half-built. Then they read `cf_cache`
  as today. They need `&mut` because rule formulas are evaluated through `evaluate_outside_cell`.
- **Callers.**
  - The engine's conditional-formatting tests and `rustsheet-xlsx/tests/test_conditional_formatting.rs`.
  - `UserModel`.
  - The Python binding, where Python callers see no change: a method that takes `PyRefMut`
    fails only on a re-entrant borrow, and nothing here re-enters.
    `bindings/python/tests/test_styles.py` must keep passing.
  - The `rustsheet` crate exposes none of these.
- **Public refresh.** `Model::evaluate_conditional_formatting` stays public for callers that want
  the work done eagerly.

### Resource limits (directive 5)
This change takes the conditional-format work out of load and `evaluate()`. It does not bound
it. A host that reads styles still pays the whole evaluation on the first read after each
`evaluate()`: 41978's rules cost about 190 s, and a hostile workbook can cost more. A `UserModel`
host that reads styles after every edit pays that per edit, as it does today.

Bounding one conditional-format evaluation (a work budget per evaluation, with a DEVIATION entry,
decision 0014) is T-0476. Evaluating only the rules whose areas hold the queried cell is a further
option for that task.

### When the evaluation happens, and what it can see
Today the cache is a snapshot taken at the end of `evaluate()`, or at load. The lazy evaluation
runs at the first style read instead. The two agree whenever nothing changes in between, which
covers the eval harness and `UserModel`, since `UserModel` evaluates after every edit, undo and
redo (`evaluate_if_not_paused`). Two differences remain, and both are accepted:
- **Edits made through `Model`, or through a paused `UserModel`, without `evaluate()`.** If cells are changed between `evaluate()`
  and the first style read, the conditional formats see the changed inputs. Today they would have
  seen the snapshot. Formula cells are not re-evaluated by this; they keep their last values, as
  today.
- **Values read between load and the first `evaluate()`.** At load, today's evaluation evaluates
  the formula cells the rules read (`evaluate_cell` outside a pass) and stores their values. So a
  value read after load could be a recomputed one rather than the file's cached one. With the
  change, values read after load are the file's cached values until `evaluate()` or a style read.
  That matches Excel, which opens a workbook showing its cached values. T-0468 records this change in
  docs/decisions/.

The eval harness never reads styles, so 41978's "under 60 s" below holds because the
conditional-format work is no longer done at all there. A style read of 41978 still costs about
190 s until T-0476.

## Alternatives considered
- **A static "this rule can never match" check** (a rule formula that is always an array). It
  fixes 41978 only, and skipping an evaluation still changes the load-time side effect above.
- **Interior mutability (`RefCell`) for the cache.** Rule evaluation needs `&mut Model`, not just
  a mutable cache.
- **A cap on conditional-format work** (directive 5). It would change styles, needs a DEVIATION
  entry, and still charges every load.

## Test plan
- These pass unchanged, with only the receivers changed: the engine's `test/conditional_formatting`,
  `rustsheet-xlsx/tests/test_conditional_formatting.rs` and `bindings/python/tests/test_styles.py`.
- New tests:
  - a load or an `evaluate()` followed by no style read evaluates no rule (a `#[cfg(test)]`
    counter of `evaluate_conditional_formatting` runs on `Model`);
  - the first read after `evaluate()` or `set_theme`, and after a `UserModel` conditional-format
    edit, undo or redo, sees the new state;
  - a second read does not re-evaluate.
- `rustsheet-eval check`: 0 regressions in every gated suite.
- corpus-bench 41978 runs under 60 s through the harness, and its `!Timeout` line leaves the
  baseline.

## Rollout
1. **T-0468 (M).** The flag, the receivers (the binding included), the four call sites, the decision
   record and the tests above, measured on 41978.
2. **T-0476 (M).** A bound on one conditional-format evaluation (directive 5).
