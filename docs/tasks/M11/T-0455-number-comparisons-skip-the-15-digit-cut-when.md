---
id: T-0455
title: "Number comparisons skip the 15-digit cut when it cannot change the answer"
milestone: M11
lane: perf
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
Comparing two numbers (`=`, `<`, MATCH, criteria, sorts: everything through
`number_format::cmp_excel_precision`) gives exactly the same answers as before, without formatting
both numbers as text when the 15-digit cut cannot change the result.

## Context
Profiling corpus-bench 59932 (one of T-0430's timeouts) showed about 80% of its 221 s
evaluation is in `cmp_excel_precision`. That workbook has 150 array formulas such as
`MIN(IF($D:$D=K2,$F:$F))`, which compare a million cells per formula. Each comparison ran
`to_excel_precision` on both numbers, a `format!` and a parse each. The VLOOKUPs T-0448 blames
for 59932 barely show in the profile.

The cut is monotone, so it can only make two numbers equal, never swap them. Two numbers it
makes equal are less than one 15-digit unit of the larger apart, at most 1e-14 of its magnitude.
Equal numbers, and numbers further apart than 1e-13 of the larger, therefore need no cut. NaN and
infinities keep the full path.

59932's #ERROR! cells are not this task's. They are whole-column VLOOKUPs into linked-workbook
sheets that the file has no cache for, which hit the array-size bound.

## Acceptance criteria
- [ ] A unit test checks the shortcut against the full cut on several hundred thousand pairs:
      neighbouring doubles, the 15th digit's rounding edge, decade boundaries, zeros, subnormals,
      infinities and NaN. A wrong threshold must fail it.
- [ ] corpus-bench 59932's release evaluation time is in the Outcome, before and after.
- [ ] `make eval` shows no new known failure; `make check` green; reviewers approve.

## Out of scope
- Building million-row arrays for whole-column operands (needs a spec; file a follow-up).
- The value of a lookup into an uncached linked-workbook sheet (file a probe follow-up).
