---
id: T-0261
title: "CONCAT: clip range iteration to the used area"
milestone: M2
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`fn_concat` walks only the used area of a range argument, as `fn_textjoin` does, so `CONCAT(A:A)` costs the used cells, not 1M empty ones. Results are unchanged.

## Context
Performance review of T-0243: CONCAT iterates every cell of a range (`right.row + 1` unchecked too) while TEXTJOIN uses `clip_to_used_area`. Empty cells add nothing to CONCAT, so clipping cannot change values. Code: `fn_concat` in functions/text/concatenate.rs; the follow-up could also share one value-to-text step between CONCAT and TEXTJOIN.

## Acceptance criteria
- [ ] CONCAT over a full-column or full-sheet range clips to the used area; a unit test covers it with unchanged values.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_concat` already walked ranges through `for_each_cell_in_range`, which clips whole-row and whole-column arguments to the used area, so no code change was needed. Added `concat_over_whole_sheet_ranges` to pin it: whole column, whole row and whole sheet join the same text as the used-area range, within the time bound. No eval delta.
