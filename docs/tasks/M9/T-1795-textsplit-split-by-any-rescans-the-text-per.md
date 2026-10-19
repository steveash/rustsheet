---
id: T-1795
title: "TEXTSPLIT split_by_any rescans the text per delimiter: cap or cache"
milestone: M9
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTSPLIT's splitting cost is bounded by the text length, not by parts x delimiters x length (CLAUDE.md directive 5).

## Context
Found by the T-1781 security review (pre-existing). `split_by_any` (textsplit.rs, about lines 260-294) rescans the rest of the text for every delimiter on every iteration, so the cost is roughly parts x delimiters x length, and the delimiter count (an array or range argument) is uncapped.

## Acceptance criteria
- [ ] `split_by_any` caches each delimiter's next match position (or otherwise scans once), so cost is near linear in text length times delimiters.
- [ ] If still needed, the delimiter count is capped with a DEVIATION entry (decision 0014).
- [ ] A test with many delimiters over a 32,767-char text finishes quickly; results are unchanged (eval: 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving TEXTSPLIT onto the SPEC-0011 driver.
