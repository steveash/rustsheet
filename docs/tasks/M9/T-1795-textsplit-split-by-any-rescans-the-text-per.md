---
id: T-1795
title: "TEXTSPLIT split_by_any rescans the text per delimiter: cap or cache"
milestone: M9
lane: functions-text
status: done
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
- [x] `split_by_any` caches each delimiter's next match position (or otherwise scans once), so cost is near linear in text length times delimiters.
- [x] If still needed, the delimiter count is capped with a DEVIATION entry (decision 0014). N/A: the existing work cap already bounds it.
- [x] A test with many delimiters over a 32,767-char text finishes quickly; results are unchanged (eval: 0 regressions).
- [x] `make check` green; reviewers approve.

## Out of scope
- Moving TEXTSPLIT onto the SPEC-0011 driver.

## Outcome
Already satisfied on main: `split_by_any` caches each delimiter's next match and re-searches only once `pos` has passed it (T-1272 performance review), so each delimiter scans the text once, and the lifted-text work cap (`text_call_cost` / `ScanCost`, text length x delimiters; T-1272, T-1988, docs/excel-semantics/text.md "Lifted TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* calls are capped") already refuses a long text with thousands of delimiters, so no new cap or DEVIATION is needed. Added the regression test `many_delimiters_over_a_long_text_are_bounded_by_the_work_cap` (32,766 commas with 200 extra delimiters gives 32,767 columns; 5,000 extra is `#ERROR!`). No engine change; eval unchanged.
