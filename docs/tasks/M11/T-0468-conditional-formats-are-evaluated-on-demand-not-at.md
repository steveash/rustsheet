---
id: T-0468
title: "Conditional formats are evaluated on demand, not at every load and recalculation"
milestone: M11
lane: perf
status: todo
priority: P2
size: M
tier: opus
depends_on: []
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

Only `get_extended_style_for_cell` (and `UserModel::get_extended_cell_style`) read `cf_cache`, and
nothing in value evaluation does. Design points to settle (a short spec first if the public receivers
change): the getters take `&self` while CF evaluation needs `&mut self` (evaluate_outside_cell); what
"the state of the last `evaluate()`" means when cells change between `evaluate()` and the first read
(today the cache is a snapshot at the end of `evaluate()`); invalidation on `set_theme` and CF edits.

## Acceptance criteria
- [ ] `Model::from_workbook` and `evaluate()` no longer evaluate conditional formats; the
      conditional-formatting tests (engine and `rustsheet-xlsx/tests/test_conditional_formatting.rs`)
      pass unchanged.
- [ ] corpus-bench 41978 loads and calculates in under 60 s through the harness; its `!Timeout`
      line leaves eval/baseline/corpus-bench.txt via `make eval-baseline`; release timings
      before/after in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making a single conditional-format evaluation over huge areas fast (a later task if a caller
  needs it).
- What Excel does with a rule formula that returns an array (keep today's behavior, which has no
  oracle evidence yet: the probe is T-2716).
