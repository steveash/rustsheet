---
id: T-0405
title: "Bound range walks: cached sheet dimensions and a cell-visit budget for 3-D spans"
milestone: M3
lane: perf
status: done
priority: P3
size: M
tier: opus
depends_on: [T-0049]
---

## Goal
A range walk costs no more than the data it reads, also for 3-D spans over many sheets.

## Context
T-0122 review (performance, security; minor): `for_each_cell_or_error_in_range`
(crates/rustsheet-engine/src/functions/util.rs) now walks a 3-D span sheet by sheet. Per sheet,
`clip_to_used_area` (evaluation.rs) calls `Worksheet::dimension()`, which scans every cell of the
sheet (T-0049 tracks a cached used area), so `SUM(S1:S50!A:A)` scans all 50 sheets on every
evaluation. And an explicit block that is neither whole-row nor whole-column
(`Data1:Data999!A2:Z1048575`) is walked cell by cell on every sheet: the pre-existing single-sheet
cost times the span's sheet count.

## Acceptance criteria
- [x] A walk visits only stored cells (or the used area) of each sheet, for explicit blocks too.
- [x] A bench case for a 3-D whole-column SUM over 50 sheets (or a deterministic visit-count test).
- [x] `make check` green; reviewers approve.

## Out of scope
- 3-D reference semantics (T-0196, T-0197, T-0198).

## Outcome
The per-sheet `Worksheet::dimension()` scan was already gone (T-0049's cached `used_area_end`).
This task clips explicit blocks too: the dense range walk (`for_each_position_in_area`, behind
every aggregate) now cuts any range, not only whole rows and columns, at each sheet's used area
(`clip_block_to_used_area`, evaluation.rs), recording the rest as read empty as before, so
`SUM(Sheet1:Sheet50!A2:Z1048575)` costs the data of 50 sheets. A block is cut only when the cut
leaves out at least 1,024 cells and the formula's seen-range budget has room for the record;
otherwise it is walked cell by cell as before, so many short blocks read by a LAMBDA never reach
the seen-range cap (review finding: `MAKEARRAY(70000,..,OFFSET($A$1,r,0,2,1))` would have been
`#VALUE!`). The two walkers that read a blank
cell (GCD/LCM read it as 0; TEXTJOIN over a 3-D span with ignore_empty FALSE adds a delimiter)
use the new `for_each_cell_or_error_in_whole_range`, which keeps walking an explicit block whole.
Tests: `test_clipped_range_walks` (3-D whole-column and block walks over 50 sheets bounded in
time, block past the used area sees later writes, a spill into a clipped block restarts, blank
readers see the whole block, 70,000 short blocks stay under the cap); a tall-block case in the
`aggregate` bench. Bench (`cargo bench -p rustsheet --bench aggregate`, medians, main → branch,
run-to-run noise about ±15%): SUM 117–152 → 128–152 ms, AVERAGE 122–151 → 131–143 ms, SUM with
an error in A2 78–92 → 82–99 ms, SUM over `A1:C1048000` 82–89 → 86–100 ms: no change beyond
noise on one sheet. Eval delta: none (performance only).
