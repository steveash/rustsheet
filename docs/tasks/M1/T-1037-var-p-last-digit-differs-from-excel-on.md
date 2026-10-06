---
id: T-1037
title: "VAR.P last digit differs from Excel on small integer data"
milestone: M1
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
VAR.P (and AGGREGATE 11) match Excel to the last digit.

## Context
Found by T-0993: `AGGREGATE(11,0,E1:E6)` over 3,1,4,1,5,9 is 7.472222222222222 in Excel and 7.472222222222221 in the engine (probes/aggregate-grid Grid!B31, C31, known failures). AGGREGATE hands the numbers to `fn_var_p` (crates/rustsheet-engine/src/functions/statistical/variance.rs), which uses the one-pass sum-of-squares form. Add a direct `VAR.P` probe of the same data first to confirm the gap is VAR.P's, then fit Excel's algorithm (it also overflows for equal huge values, numbers.md "Overflow": not a stable two-pass deviation; T-1029).

## Acceptance criteria
- [ ] probes/aggregate-grid B31, C31 pass and a direct VAR.P probe of the same data passes, with no regressions in other VAR.P cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
