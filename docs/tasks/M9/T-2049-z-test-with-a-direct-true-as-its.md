---
id: T-2049
title: "Z.TEST with a direct TRUE as its array is one value (#DIV/0!), not #VALUE!"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
`Z.TEST(TRUE,0.5,1)` (and ZTEST) is `#DIV/0!` as in Excel, not `#VALUE!`.

## Context
- Known failure (gap triage 2026-10-19): probes-numeric/t1049-ztest Calc!A51 `Z.TEST(TRUE,0.5,1)`:
  Excel (excel-graph) `#DIV/0!`, ours `#VALUE!`. The same case shows a direct number is one value and
  one value is `#DIV/0!` even with sigma (A41 `Z.TEST(7,4)`, A42 `Z.TEST(7,4,1)`, both `#DIV/0!`, and
  pass), while direct text is `#VALUE!` (A50 `Z.TEST("a",4)`, passes). So a direct boolean data
  argument is accepted like a number (whether Excel counts it or skips it, the result is
  `#DIV/0!`; the evidence cannot tell, and both give the same value).
- Code: `z_test_data` in crates/rustsheet-engine/src/functions/statistical/z_test.rs matches
  `CalcResult::Number` but sends `CalcResult::Boolean` to the `#VALUE!` arm.
- docs/excel-semantics/statistics.md: "Z.TEST with a single-value array returns #DIV/0!" (line ~473)
  and the "RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments" entry.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cell, 0 numeric past half a unit in the 15th digit and 1 with a wrong type, error or text, blamed on Z.TEST 1. Every one is listed above.

## Acceptance criteria
- [ ] Visible parity: the visibly-off cell listed under Context (2026-10-21) has Excel's value to 15 significant digits, or its type, error and text: the 1 cell of the ungated probes-numeric suite passes in `make eval`, with no new known-failure lines in any baseline.
- [ ] probes-numeric/t1049-ztest Calc!A51 passes (`make eval`, the ungated probes-numeric suite), and
      no other Z.TEST/ZTEST cell changes in any suite (no new known-failure lines).
- [ ] A unit test covers `Z.TEST(TRUE,0.5,1)` and `ZTEST(TRUE,0.5,1)` (`#DIV/0!`; ZTEST equals Z.TEST, statistics.md legacy-names entry).
- [ ] statistics.md records the direct-boolean rule with the cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Direct numeric text as the data argument (unprobed; file a probe task if needed).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (`#VALUE!` where Excel is `#DIV/0!`), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it.

## Outcome
A direct boolean data argument of Z.TEST/ZTEST is read as one value, so `Z.TEST(TRUE,0.5,1)` is `#DIV/0!`. Unit test added; t1049-ztest A51 now passes.
