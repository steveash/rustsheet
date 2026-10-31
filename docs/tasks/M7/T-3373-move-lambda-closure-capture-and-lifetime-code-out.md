---
id: T-3373
title: "Move LAMBDA closure capture and lifetime code out of logical/lambda.rs (947 of its 1,000 lines)"
milestone: M7
lane: functions-logic
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`crates/rustsheet-engine/src/functions/logical/lambda.rs` has room under the 1,000-line file limit again
(`scripts/check_file_size.py`). The code that decides what an escaping LAMBDA captures and when a formula's LAMBDAs are
dropped moves, unchanged, into a new `logical/lambda_scope.rs`, so the open LAMBDA work (T-3111's near-zero root snap,
which edits the call path) does not first have to split the file.

## Context
Sonnet refill 2026-10-31 (`make eval` on origin/main): the visibly-off LAMBDA cells (probes/t3095-snap-lambda-helpers,
65 cells) are T-3111's, and `make unowned-off ALL=1` lists no logic cell, so no functions-logic eval gap is left to cut.
lambda.rs is at 947 lines and not in quality/oversized-files.json. T-3313 set the pattern (a submodule, no behaviour
change); `logical/let.rs`, `read_once.rs` and `scan.rs` are the siblings.

What moves (line numbers at origin/main 2026-10-31):
- `trait ScopeResult` and its three impls for `CalcResult`, `RefArg` and `Binding` (l. 68-112);
- `MAX_CAPTURED_CELLS` (l. 172-178) with its doc comment (it cites the DEVIATION in arrays.md);
- `Model::close_over`, `Model::bind_captures`, `Model::unbind_captures` (l. 724-819);
- the lifetime block at the end: `RETAINED_LAMBDA_SLOTS` and the `impl Model` with `evaluate_root_cell`,
  `clear_formula_scratch`, `drop_lambdas_since` and `clear_lambdas` (l. 900-947).
`logical/mod.rs` re-exports `ScopeResult` (used by let.rs and `external_reference.rs`); keep that path working with a
`pub(crate) use`. `bind_captures`/`unbind_captures` are called from `call_prepared` in lambda.rs: widen them to
`pub(super)`, no further. Callers in evaluation.rs, data_tables.rs and nested_array.rs's doc link must not change.

## Acceptance criteria
- [ ] The items above, with their doc comments (the evidence and DEVIATION they cite), live in `logical/lambda_scope.rs`;
      their bodies are not changed (a reviewer can diff them line by line).
- [ ] lambda.rs is at most 780 lines.
- [ ] `make eval-check` (or the full eval comparison of main against HEAD) shows no cell changed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what LAMBDA, LET or the helpers compute (T-3111 owns the snap).
- The call path (`call_prepared`, `prepare_lambda`) and the eta-name code.
