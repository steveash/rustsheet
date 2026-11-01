---
id: T-3789
title: "PERCENTRANK*: a text or error significance beats the x-not-found or no-number #N/A (gen-m9, 63 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
PERCENTRANK, PERCENTRANK.INC and PERCENTRANK.EXC return the significance's error, or `#VALUE!` for a significance of text
that does not read as a number (including empty text), before the `#N/A` of an x outside the data or of data with no number, as Excel does.

## Context
- T-3781 (gen-m9 arg-pairs batch, excel-graph 2026-11-01). `PERCENTRANK.INC(Data!H1,2,"abc")` (blank data) and
  `PERCENTRANK.EXC(Data!A$5,-1,"")` are `#VALUE!`; `PERCENTRANK(Data!G5,3,Data!G9)` (text data, G9 `#DIV/0!`) is `#DIV/0!`;
  `PERCENTRANK(Data!C8,"2.5",#NAME?)` is `#NAME?`, `PERCENTRANK.EXC(Data!H1,1,#NULL!)` `#NULL!`. Ours: `#N/A` for all of them.
- This settles statistics.md's UNVERIFIED line under "Order statistics: the scalar argument's error before the no-number
  result; PERCENTRANK's #N/A before significance (T-2256)": a numeric but out-of-domain significance (-2.5, 0) still loses
  to the `#N/A` (T-2472 matches that), but a non-numeric one wins. Recorded in statistics.md "PERCENTRANK: a text or error
  significance before the #N/A (T-3781)".
- Numeric text and booleans coerce and keep the `#N/A` first, and those cells pass today: `PERCENTRANK.INC(Data!H1,2,"2.5")`,
  `(Data!H2,3,Data!G5)` (G5 = "12"), `(Data!H1,2,TRUE)` and `(Data!H2,3,Data!G8)` (G8 = FALSE) are `#N/A` (8cbdca3221390122
  A433, A436, A451, A454), as is `PERCENTRANK(Data!G5,2,Data!G8)` (de40f308bf65d545 A208). Keep them passing.
- Code: percentile.rs `percentrank_of` (x, then empty data, then significance). Decide whether the significance is a number (an error or
  non-numeric text fails here) before the x / data checks and keep the domain check where it is. T-2605 (single boolean cell, bad significance on empty data) is
  the neighbour; check its cells still pass.

## Acceptance criteria
- [ ] The 63 cells below pass, and the T-2472 cells (numeric bad significance after `#N/A`) still pass.
- [ ] statistics.md: the UNVERIFIED line is replaced by the rule, and the new entry's Ours line updated.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0159: the task is done when the gen-m9 cells below (T-3781's arg-pairs batch, excel-graph, 2026-11-01) pass and
their known-failure lines leave eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure line in any baseline.
- gen-m9/50594c61def0ea70 Calc!A46, A109, A361
- gen-m9/58ae70ab5eb596c2 Calc!A100, A157
- gen-m9/8cbdca3221390122 Calc!A142, A148, A160, A190, A196, A208, A256, A262, A274, A316, A322, A334, A439, A442, A445, A448, A457, A460, A508, A541
- gen-m9/9240e43366ea4792 Calc!A214, A220, A232, A274, A280, A292, A340, A346, A358, A400, A406, A418, A523, A526, A529, A532, A541, A544
- gen-m9/c149b14aa61b770d Calc!A10, A82
- gen-m9/de40f308bf65d545 Calc!A196, A202, A214, A250, A256, A268, A316, A322, A334, A376, A382, A394, A487, A490, A493, A496, A505, A508

## Out of scope
- A text x or significance against an error data cell (T-3788).
