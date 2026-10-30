---
id: T-3123
title: "TEXTJOIN ignore_empty FALSE single-walk path: stop reading at the length limit"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXTJOIN with ignore_empty FALSE stops reading cells once the 32,767-unit limit is passed on the unclipped single-walk path too, so a self-including range is `#CALC!` like Excel.

## Context
Follow-up of T-3098. Excel stops reading past the limit (probes/textjoin-whole-rows-self-reference B5 is `#CALC!`, not circular; docs/excel-semantics/text.md). `textjoin_push_padded_range` stops in its per-row loop, but its `skipped_columns == 0` branch and the non-padded arm of `textjoin_walk` (concatenate.rs) still read every cell. Probe a self-including range whose used area spans every column first.

## Acceptance criteria
- [ ] A probe pins a self-including whole-row TEXTJOIN with used columns spanning the range.
- [ ] The single-walk path stops reading at the limit if Excel does; read-count test added; text.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Picked by a sonnet run and handed off before any code: the task starts with an Excel probe (a self-including
whole-row TEXTJOIN whose used columns span the range, modelled on probes/textjoin-whole-rows-self-reference
B5/D3), and the engine change plus a read-count test depend on its result, which the run's remaining budget
could not cover. `rustsheet-oracle status` showed excel-graph OK on 2026-10-30. Next: use the excel-probe skill
to write and run the probe, then follow the acceptance criteria (`textjoin_walk` non-padded arm and the
`skipped_columns == 0` branch of `textjoin_push_padded_range` in concatenate.rs).

Resumed by an opus run, which found #2324 (branch agent/functions-text/T-3123-textjoin-single-walk) already
implementing this task with the probe, green CI and an approved panel; the run drives #2324 to merge, and this
branch (#2447) is the duplicate to close afterwards.
