---
id: T-1034
title: "LARGE/SMALL round a fractional k up and accept a boolean k"
milestone: M1
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
LARGE and SMALL (and AGGREGATE 14/15, which call them) treat k as Excel does.

## Context
Found by T-0993. Excel: `AGGREGATE(14,0,E1:E6,2.9)` is 4 with E1:E6 = 3,1,4,1,5,9, the 3rd largest (k rounds up), and `AGGREGATE(14,0,E1:E6,TRUE)` is 9 (TRUE is 1). The engine truncates k (gives 5) and rejects TRUE (`#VALUE!`). probes/aggregate-args Misc!A11, A13 (known failures). `fn_large`/`fn_small` in crates/rustsheet-engine/src/functions/statistical/count_and_average.rs. Probe LARGE/SMALL directly too (k as text, as a referenced boolean, 0.5, negative fractions) before changing them.

## Acceptance criteria
- [ ] probes/aggregate-args Misc!A11 and A13 pass; new LARGE/SMALL probe cells with oracle values pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
