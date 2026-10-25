---
id: T-0468
title: "Conditional formats are evaluated on demand, not at every load and recalculation"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0475]
---

## Goal
Loading and recalculating a workbook no longer pays for its conditional formats: they are
evaluated when a cell's conditional style is first asked for, and the styles returned are the ones
the eager evaluation returns today.

## Context
Found by T-0430 profiling corpus-bench `41978/1_41978_input.xlsx` (sheet A1:M185): load 184 s,
`evaluate()` another 191 s, both inside `evaluate_conditional_formatting`
(conditional_formatting.rs), which `Model::from_workbook*`, `new_empty.rs`, `set_theme` and the end
of `evaluate()` (evaluation.rs) all call. The file's expression rules cover near-million-row areas
(`G186:G1047468` with `IF(J195:J619, "open")`; `G1047889:G1048576` with `IF(J1:J1047898, "open")`),
and `apply_cf_formula` evaluates the formula once per cell of the area: about 1.6e9 array elements
per run, all discarded (an array result counts as no match, `_ => false`).

Design: docs/specs/SPEC-0021-conditional-formats-on-first-style-read.md (T-0475). Only
`get_extended_style_for_cell` (and `UserModel::get_extended_cell_style`) read `cf_cache`, and
nothing in value evaluation does. Design points to settle (a short spec first if the public receivers
change): the getters take `&self` while CF evaluation needs `&mut self` (evaluate_outside_cell); what
"the state of the last `evaluate()`" means when cells change between `evaluate()` and the first read
(today the cache is a snapshot at the end of `evaluate()`); invalidation on `set_theme` and CF edits.

## Acceptance criteria
- [x] `Model::from_workbook` and `evaluate()` no longer evaluate conditional formats; the
      conditional-formatting tests (engine and `rustsheet-xlsx/tests/test_conditional_formatting.rs`)
      pass unchanged.
- [x] corpus-bench 41978 loads and calculates in under 60 s through the harness; its `!Timeout`
      line leaves eval/baseline/corpus-bench.txt via `make eval-baseline`; release timings
      before/after in the Outcome.
- [x] The change in values read between load and the first `evaluate()` (SPEC-0021) is recorded
      in docs/decisions/; the Python binding's `get_extended_cell_style` follows the receiver.
- [x] `make eval-check` shows whether any result changes now that load stores no recomputed
      values (cycle members, iteration starts; SPEC-0021 "What `evaluate()` starts from"); any
      change is in the Outcome and the decision record.
- [x] `make check` green; reviewers approve.

## Out of scope
- Making a single conditional-format evaluation over huge areas fast (a later task if a caller
  needs it).
- What Excel does with a rule formula that returns an array (keep today's behavior, which has no
  oracle evidence yet: the probe is T-2716).

## Outcome
- SPEC-0021 as specified: `Model::cf_stale` replaces the four eager `evaluate_conditional_formatting()`
  calls (both load sites, the end of `evaluate()`, `set_theme`). `Model::get_extended_style_for_cell`,
  `UserModel::get_extended_cell_style` and the Python binding's method take `&mut self` and evaluate
  the conditional formats on the first read while the flag is set (`#[cfg(test)]` `cf_runs` counts
  the evaluations). Breaking change for Rust callers of `rustsheet-engine`: those receivers.
- corpus-bench 41978 (release): load 184 s -> 5 ms, `evaluate()` 191 s -> 6 ms. Its `!Timeout` line
  left eval/baseline/corpus-bench.txt (`make eval-baseline`: fixed 1; its 24 cells all match Excel).
- Eval: 0 regressions in every gated suite, the circular and iteration fixtures included, so no
  result changed with `evaluate()` starting from the file's cached values (decision 0117).
- `rustsheet_xlsx::compare` no longer relied on the load-time side effect: templates/wordle.xlsx
  `Wordle!N2` (`TODAY()`) had matched only because loading recomputed it on the system clock. A
  workbook without a METADATA timestamp is now pinned to its `dcterms:modified` time, on which N2
  is Excel's cached "RHINO".
- Tests: `test/conditional_formatting/on_first_read.rs` (no evaluation without a read, one
  evaluation per read, `set_theme`, `UserModel` edit/undo/redo, paused edits with and without an
  earlier read, a formula written since `evaluate()` computed by the read); the existing
  conditional-format tests with `&mut` receivers.
- Follow-up: T-0476 (a bound on one conditional-format evaluation; a style read of 41978 still costs
  about 190 s).
