---
id: T-3098
title: "TEXTJOIN ignore_empty FALSE over whole rows: break at the length limit, split the Range arm, and probe 1:3 with an error cell"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTJOIN with ignore_empty FALSE over whole rows stops walking once the 32,767-unit limit is passed, its
Range arm is split into helpers, and its unprobed `1:3`-with-error result is pinned by Excel.

## Context
Review follow-ups of T-2713. In `textjoin_walk` (crates/rustsheet-engine/src/functions/text/concatenate.rs)
the per-row loop for clipped columns keeps calling the walker for every remaining row after `over_limit`
is set (about 1M calls for `A:Z` with data far down). The arm also clips twice (`clip_to_used_area` here
and in the walker) and is about 90 lines at 5 nesting levels. docs/excel-semantics/text.md records
`TEXTJOIN(",",FALSE,1:3)` with `=NA()` in A3 giving `#CALC!` as unprobed, and
`textjoin_pads_clipped_columns_after_each_row` (test_aggregate_sparse_walk.rs) locks it in. The
ignore_empty FALSE clipped-column path has no read-count assertion.

## Acceptance criteria
- [ ] The per-row loop breaks once `over_limit` is set; the result is unchanged.
- [ ] The Range arm is split into helpers (for example `textjoin_push_padded_range`) with one clip.
- [ ] A work-counter test covers the ignore_empty FALSE clipped-column path.
- [ ] A probe case (excel-probe skill) pins `TEXTJOIN(",",FALSE,1:3)` with an error cell; text.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXTJOIN over unions and the 32,767-unit limit rules.
