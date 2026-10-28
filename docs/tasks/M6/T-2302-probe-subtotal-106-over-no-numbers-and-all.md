---
id: T-2302
title: "Probe SUBTOTAL 106 over no numbers (and all-hidden numeric rows)"
milestone: M6
lane: functions-math
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for SUBTOTAL(106,...) over text/blank cells and over a range whose numeric rows are all hidden: 0 or 1.

## Context
T-2269 made function_num 6 give 0 over no numbers (verified); 106 shares the code path (`subtotal_product`) but is UNVERIFIED. See docs/excel-semantics/numbers.md "SUBTOTAL 6 over no numbers is 0"; use the excel-probe skill.

## Acceptance criteria
- [x] Probe case with Excel values committed; numbers.md entry updated to VERIFIED or corrected, with code and test following the evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/subtotal-106-no-numbers` (16 excel-graph cells): SUBTOTAL 106 over text only, blanks only,
and ranges whose only numbers are manually hidden or table-filtered is 0, never 1; AGGREGATE 6 under
options 1/4/5/6 is 0 the same way; SUBTOTAL 6 still multiplies manually hidden numbers (15) but skips
filtered ones (0). The engine already matched (16/16 pass, no code change); numbers.md entry now VERIFIED.
Eval delta: +16 passing cells (new case), no regressions.
