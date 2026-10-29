---
id: T-3123
title: "TEXTJOIN ignore_empty FALSE single-walk path: stop reading at the length limit"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
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
