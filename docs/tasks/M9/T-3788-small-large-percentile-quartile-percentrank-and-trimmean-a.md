---
id: T-3788
title: "SMALL, LARGE, PERCENTILE*, QUARTILE*, PERCENTRANK* and TRIMMEAN: a text k, x or significance is #VALUE! before an error cell as the data (gen-m9, 30 cells)"
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
SMALL, LARGE, PERCENTILE*, QUARTILE*, TRIMMEAN and PERCENTRANK* return `#VALUE!` for a text k, quart, percent, x or
significance (a cell holding text or empty text) when the data is a one-cell reference to an error cell, as Excel does;
a numeric scalar argument still lets the data's error through.

## Context
- T-3781 (gen-m9 arg-pairs batch, excel-graph 2026-11-01). Data!G9 holds `#DIV/0!`, G6 the text "x", G7 empty text.
  `SMALL(Data!G9,Data!G6)`, `LARGE(Data!G9,Data!G7)`, `PERCENTILE.EXC(Data!G9,Data!G6)`, `QUARTILE(Data!G9,Data!G7)`,
  `TRIMMEAN(Data!G9,Data!G6)`, `PERCENTRANK(Data!G9,Data!G6)` and `PERCENTRANK(Data!G9,2,Data!G6)` are `#VALUE!` in Excel and
  `#DIV/0!` in ours. With a number as the scalar the data's error wins (`LARGE(<#REF!>,0.1)` is `#REF!`; statistics.md "Array
  arguments given a scalar, a single cell or an error").
- Recorded in docs/excel-semantics/statistics.md "A text scalar argument is #VALUE! before an error cell as the data (T-3781)".
  Unprobed: an error-valued k against an error data cell (which error wins), and the data given as a multi-cell range
  holding an error (an error element, not a lone cell).
- Code: count_and_average.rs (`fn_large`, `fn_small`), percentile.rs (`percentile_numbers`, `percentrank_of`), quartile.rs,
  trimmean.rs. Find the one place each reads the data before the scalar and share the fix (`rg` first; CLAUDE.md directive 3).

## Acceptance criteria
- [ ] The 30 cells below pass.
- [ ] Unit or probe cells pin both orders: a text scalar with an error data cell (`#VALUE!`), a numeric scalar with an
      error data cell (the data's error).
- [ ] statistics.md entry's Ours line updated.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0159: the task is done when the gen-m9 cells below (T-3781's arg-pairs batch, excel-graph, 2026-11-01) pass and
their known-failure lines leave eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure line in any baseline.
- gen-m9/157db6a46ed6535b Calc!A454, A460
- gen-m9/3df602816eacaa3f Calc!A424, A430
- gen-m9/45d8028602e891ad Calc!A442, A448
- gen-m9/4740a028a32c91de Calc!A418, A424
- gen-m9/475eae18f2963cdc Calc!A466, A472
- gen-m9/4fc6bdaec0e5583d Calc!A478, A484
- gen-m9/5830c688df805e71 Calc!A466, A472
- gen-m9/5e5063d014818a2d Calc!A460, A466
- gen-m9/61445b60bf8539f0 Calc!A430, A436
- gen-m9/8cbdca3221390122 Calc!A382, A388
- gen-m9/9240e43366ea4792 Calc!A466, A472
- gen-m9/a447db3e2a26d5ed Calc!A430, A436
- gen-m9/d7ee86e14cbfbcf3 Calc!A460, A466
- gen-m9/de40f308bf65d545 Calc!A442, A448
- gen-m9/f00b7d6982c4d2fd Calc!A430, A436

## Out of scope
- PERCENTRANK's significance against the x-not-found `#N/A` (T-3789); TRIMMEAN of a typed scalar (T-3790).
