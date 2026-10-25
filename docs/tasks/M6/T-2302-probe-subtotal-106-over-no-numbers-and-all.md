---
id: T-2302
title: "Probe SUBTOTAL 106 over no numbers (and all-hidden numeric rows)"
milestone: M6
lane: functions-math
status: in-progress
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
- [ ] Probe case with Excel values committed; numbers.md entry updated to VERIFIED or corrected, with code and test following the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
