# SPEC-0021 — Conditional formats evaluated on the first style read

- **Status:** accepted
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
  (new_empty.rs). It is set at exactly the four call sites of `evaluate_conditional_formatting()`
  today, which no longer call it:
  1. load in `Model::from_workbook*` (model.rs), which covers every rustsheet-xlsx import;
  2. load in new_empty.rs;
  3. the end of `evaluate()` (evaluation.rs);
  4. `set_theme` (model.rs).

  `UserModel`'s edits evaluate (`evaluate_if_not_paused`), and so do its undo and redo, so they set
  the flag. No setter site is added.
- **Mutable style reads.** These public methods change from `&self` to `&mut self`:
  - `Model::get_extended_style_for_cell(&mut self, ..)`;
  - `UserModel::get_extended_cell_style(&mut self, ..)`;
  - the Python binding's `UserModel.get_extended_cell_style` (bindings/python/src/user_model.rs),
    which becomes a `&mut self` pyo3 method.

  When the flag is set they call `evaluate_conditional_formatting()`, then clear it. That function
  returns nothing and cannot fail, so the cache is never left half-built. Then they read `cf_cache`
  as today. They need `&mut` because rule formulas are evaluated through `evaluate_outside_cell`.
- **Callers.**
  - Engine tests: `test/conditional_formatting`, `test/test_language_switch.rs`,
    `test/user_model/test_language_switch.rs` and `test/user_model/test_conditional_formatting.rs`.
  - `rustsheet-xlsx/tests/test_conditional_formatting.rs`.
  - `UserModel`.
  - The Python binding, where Python callers see no change: a method that takes `PyRefMut`
    fails only on a re-entrant borrow, and nothing here re-enters.
    `bindings/python/tests/test_styles.py` must keep passing.
  - The `rustsheet` crate exposes none of these.
- **Breaking change.** The receiver change breaks Rust callers of `rustsheet-engine`. The T-0468
  PR lists it in its description.
- **Public refresh.** `Model::evaluate_conditional_formatting` stays public for callers that want
  the work done eagerly.

### Resource limits (directive 5)
This change takes the conditional-format work out of load and `evaluate()`. On its own it did not
bound it: the first style read after each `evaluate()` paid the whole evaluation (41978: about
190 s).

Revised by T-0476: a style read evaluates only the rules whose areas hold the cell read, in
priority order, for that cell (`cf_styles.rs`, `CfState`). A rule's statistics over its area (its
numbers, or its value counts for the unique and duplicate rules) are computed once per round,
over the area cut at the used area, where every cell left out is empty. Styles are those of the
evaluation of every cell, which `evaluate_conditional_formatting()` still does for a caller who
asks for it. A read now costs the rules over its cell, each one formula at most, plus once per
round the statistics over the used area. 41978's first eight reads take 322 ms, rows 1,047,886
and beyond included, and a read of all 2,405 used cells takes 152 ms. Styles do not change, so
there is no DEVIATION.

### When the evaluation happens, and what it can see
Today the cache is a snapshot taken at the end of `evaluate()`, at load, or at `set_theme`. The
lazy evaluation runs at the first style read after one of those instead, and sees the workbook as
it is then. The two agree whenever nothing changes in between. That covers the eval harness (which
never reads styles) and `UserModel` while it is not paused, since `UserModel` evaluates after every
edit, undo and redo.

Once a read has cleared the flag, each cell's results stay as they are until the next of the four
call sites, exactly as today's snapshot does. Since T-0476 a cell's rules run at that cell's first
read in the round, so what follows holds for each cell's first read, not only the round's first. The differences all come from changes made through `Model`, or
through a paused `UserModel`, after the snapshot point but before the first read. All are accepted:
- **The read sees those edits.** It sees edited cells, and edited rules, dxfs, rows, columns and
  sheets (`Model::add_conditional_formatting` and the other CF edits, undo/redo and clipboard
  writes to `Worksheet::conditional_formatting`, `create_dxf`). Today the snapshot would not.
- **The read can evaluate formula cells.** A rule reads cells through `evaluate_cell`, outside a
  pass, so:
  - a formula written since the last `evaluate()` at a position that pass did not evaluate (an
    empty or constant cell then) is computed by the read and its value stored, spill included;
  - a formula written at a position that pass did evaluate is read through `stored_value`, which
    gives `#ERROR!` for a cell that has no value yet (cell.rs).

  Today's snapshot runs right after `evaluate()`, so it meets neither case. A style read can thus
  change a value read afterwards, in the first case, where today none can.
- **Values read between load and the first `evaluate()`.** At load, today's evaluation evaluates
  the formula cells the rules read (`evaluate_cell` outside a pass, while `evaluation.cells` is
  still empty) and stores their values. So a value read after load could be a recomputed one rather
  than the file's cached one. With the change, values read after load are the file's cached values
  until `evaluate()` or a style read. That is the usual behavior of a workbook opened without
  recalculation, though no oracle case pins it. T-0468 records this change in docs/decisions/.
- **What `evaluate()` starts from.** Since load no longer stores those recomputed values,
  `evaluate()` starts from the file's cached values. Some results keep the values they start from:
  the members of a cycle (docs/excel-semantics/recalc.md, "keeps cached values") and the start of
  iterative calculation. Those results can therefore change, presumably towards Excel's, which
  starts from the same cached values. T-0468 checks this with `make eval-check`, the fixtures with
  circular references (circular-keeps-cached-values) included, and records any change.

The eval harness never reads styles, so 41978's "under 60 s" below holds because the
conditional-format work is no longer done at all there. A style read of 41978 still costs about
190 s until T-0476.

## Alternatives considered
- **A static "this rule can never match" check** (a rule formula that is always an array). It
  fixes 41978 only, and skipping an evaluation still changes the load-time side effect above.
- **Keeping a `&self` read that returns the cache without refreshing it.** That gives a caller
  stale or empty styles silently; one refreshing method is simpler. Callers that want the work
  done ahead of time call `evaluate_conditional_formatting()`.
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
  - under paused evaluation, after a style read has cleared the flag, a conditional-format edit and
    its undo leave the read styles unchanged until `evaluate()`, as today's snapshot does;
  - without that earlier read, the first read sees the edit (the accepted difference above);
  - a formula written through `Model` after `evaluate()`, then a style read whose rule reads it,
    then a value read: the value is the one the read computed (an unevaluated position), or
    `#ERROR!` (a position the pass evaluated), as described above;
  - a second read does not re-evaluate.
- `rustsheet-eval check`: 0 regressions in every gated suite.
- corpus-bench 41978 runs under 60 s through the harness, and its `!Timeout` line leaves the
  baseline.

## Rollout
1. **T-0468 (M).** The flag, the receivers (the binding included), the four call sites, the decision
   record and the tests above, measured on 41978.
2. **T-0476 (M).** A bound on one conditional-format evaluation (directive 5).
