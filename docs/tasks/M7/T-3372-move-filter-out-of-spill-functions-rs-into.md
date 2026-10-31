---
id: T-3372
title: "Move FILTER out of spill_functions.rs into its own module (spill_functions.rs is at 952 of its 1,000 lines)"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`crates/rustsheet-engine/src/functions/spill_functions.rs` has room under the 1,000-line file limit again
(`scripts/check_file_size.py`). FILTER moves, unchanged, into a new `lookup_and_reference/filter.rs`, beside the other
M7 array functions (`drop_take.rs`, `hstack_vstack.rs`, `tocol_torow.rs`), so the open SORT/SORTBY/UNIQUE work (T-3211,
collation) has room in spill_functions.rs.

## Context
Sonnet refill 2026-10-31 (`make eval` on origin/main): every visibly-off lookup cell has an open owner (the t3275 and
t3181 collation cells: T-3321, T-3210, T-3211; `make unowned-off ALL=1` lists none), so no functions-lookup eval gap is
left to cut. spill_functions.rs is at 952 lines, 48 under the limit, and not in quality/oversized-files.json; T-3211's
collation sort will add to it. T-3313 set the pattern (move code into a submodule, no behaviour change).

What moves (line numbers at origin/main 2026-10-31):
- the `// ── FILTER` section to the end of the `impl Model` block: `Model::fn_filter` with its doc comment
  (l. 812-952);
- `array_node_is_truthy` (l. 54-60), which only FILTER calls.
`fn_filter` uses `Model::eval_array_arg` and friends, which stay in spill_functions.rs (`pub(crate)` already). The
dispatch in `functions/mod.rs` (`Function::Filter => Self::fn_filter`) does not change, since `fn_filter` stays a
`Model` method. Declare the module in `lookup_and_reference/mod.rs` with the others.

## Acceptance criteria
- [ ] `fn_filter` and `array_node_is_truthy`, with their comments, live in `lookup_and_reference/filter.rs`; the code is
      not changed (a reviewer can diff it line by line).
- [ ] spill_functions.rs is at most 830 lines.
- [ ] `make eval-check` (or the full eval comparison of main against HEAD) shows no cell changed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what FILTER computes. (Its doc comment's note that a comparison like `A1:A5>3` must be stored in a
  helper column first looks stale; the moved comment may drop that sentence, nothing else.)
- SORT, SORTBY and UNIQUE (T-3211 owns their collation work).
