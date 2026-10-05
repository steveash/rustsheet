---
id: T-0955
title: "OFFSET: checked arithmetic for huge rows/cols/height/width (i32 overflow panics in debug)"
milestone: M9
lane: functions-lookup
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`OFFSET` with huge rows/cols/height/width (e.g. `OFFSET(A1,1E10,0)`, `OFFSET(A1,1,0,1E10)`) returns `#REF!` instead of overflowing i32 (panics in debug, wraps in release).

## Context
Found by the T-0946 security review. `fn_offset` (functions/lookup_and_reference/mod.rs) casts f64 with `as i32` (saturating) then does unchecked `left.row + rows`, `row_start + height`, `column_start + width`. Directive 5: no panics on untrusted input. Excel gives #REF! for out-of-sheet results.

## Acceptance criteria
- [x] Unit tests for the huge-argument forms above return `#REF!` in debug builds; arithmetic uses checked/saturating ops.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`fn_offset` and `offset_size` use saturating arithmetic, so huge rows/cols/height/width give `#REF!` (negative height/width keep their existing `#VALUE!`) instead of an i32 overflow panic. Test `fn_offset_huge_arguments_are_ref_errors` panics before the fix. No eval change.
