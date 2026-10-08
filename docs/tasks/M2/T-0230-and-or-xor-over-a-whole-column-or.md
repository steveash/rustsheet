---
id: T-0230
title: "AND/OR/XOR over a whole-column or whole-sheet range walks every cell: clip to the used area"
milestone: M2
lane: functions-logic
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
AND/OR/XOR over a whole-column or whole-sheet range stay bounded (clip the walk to the used area) with Excel's results unchanged.

## Context
After T-0215 removed the early return in `logical_nary`, `AND(A:XFD)`-style calls always walk every cell; XOR already did. Untrusted input must not trigger unbounded work (CLAUDE.md directive 5). Cells beyond the used area are empty and ignored, so clipping is safe; mind `record_seen_empty_range` dependency tracking as in `fn_countifs`.

## Nightly fuzz crash (2026-10-08)
Nightly fuzz run 37724893331 (`main` b38d244), target `formula_eval`, artifact
`oom-b4c16080061986ca796d91c523aa48d88ffb2e67`: out of memory (rss limit 2048 MB) on the formula

    OR(IFERROR(ERROR.TYPE(TRUNC($Z$1))=5,F:LSE))

(bytes: `4f 52 28 49 46 45 52 52 4f 52 28 45 52 52 4f 52 2e 54 59 50 45 28 54 52 55 4e 43 28 24 5a 24 31 29 29 3d 35 2c 46 3a 4c 53 45 29 29`).
`IFERROR` returns the reference `F:LSE` (8,606 whole columns) and `logical_nary` calls `evaluate_cell` on every one of its cells. Reproduced on `main` bccce59 (release build): `OR(F:H)` takes 0.5 s and 277 MB peak RSS, `OR(F:AZ)` 23 s and 4.3 GB, so memory grows with every empty cell visited, not only time. Raised to P0 and moved from `perf` to `functions-logic` (the walk lives in `functions/logical/and_or_xor_not.rs`) by the 2026-10-08 steward run (decision 0035).

## Acceptance criteria
- [x] The reproducer above is a unit test (decision 0035) that finishes in well under a second with bounded memory and returns the same result as `OR(F:LSE)` (today `#VALUE!`: no logical values in an empty range).
- [x] Whole-sheet AND/OR/XOR completes quickly; results unchanged; dependency tracking still correct; `make check` green.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Scope note (from the T-0225 review)
The same unbounded walk exists in every `*_or_array` lift (ABS, TRIM, NOT, ...) because `Model::evaluate_range`
(`model.rs`) materializes every cell of the range, so `=NOT(A:XFD)` tries ~17 billion elements. Fix it in
`evaluate_range` (clip to the used area, keep `record_seen_empty_range` tracking) rather than per function.

## Outcome
`logical_nary` (AND/OR/XOR) now walks a range argument through `for_each_cell_or_error_in_range`,
which clips whole-row/whole-column ranges to the used area and records the skipped rectangle as
read empty (so a later value there still triggers a recalc). The fuzz reproducer
`OR(IFERROR(ERROR.TYPE(TRUNC($Z$1))=5,F:LSE))` is a unit test (`test_fn_or_xor.rs`) and returns
`#VALUE!` instantly; whole-sheet / whole-column / whole-row AND/OR/XOR equal their used-area
results (`test_clipped_range_walks.rs`). The scope note's `evaluate_range` clipping had already
landed (NOT and the other lifts). Explicit near-whole blocks (`A1:XFD1048575`) remain unclipped in
every walker; T-0405 tracks that. Eval delta: none (no eval cell changes).
