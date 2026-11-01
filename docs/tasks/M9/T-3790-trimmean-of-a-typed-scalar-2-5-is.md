---
id: T-3790
title: "TRIMMEAN of a typed scalar: \"2.5\" is 2.5 and TRUE is 1, the percent's own result or error follows (gen-m9, 15 cells)"
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
TRIMMEAN reads a typed scalar data argument as Excel does: `TRIMMEAN("2.5",0)` is 2.5 and `TRIMMEAN(TRUE,0)` is 1, a bad
percent is then `#NUM!` and an error percent is that error.

## Context
- T-3781 (gen-m9 arg-pairs batch, excel-graph 2026-11-01), case 4fc6bdaec0e5583d: `TRIMMEAN("2.5",0)`,
  `TRIMMEAN("2.5",Data!H1)` (blank) are 2.5 (so is the top-left cell of `TRIMMEAN("2.5",Data!A1:B2)`), the TRUE forms 1; `TRIMMEAN("2.5",2)`,
  `(..,-1)`, `(..,"2.5")` and `(..,TRUE)` `#NUM!`; `TRIMMEAN("2.5",#NULL!)` `#NULL!`, `TRIMMEAN(TRUE,#REF!)` `#REF!`.
  Ours: `#VALUE!` for every one. A typed numeric-text or boolean literal is one number here, unlike a cell holding text or a
  boolean (no number, statistics.md "A lone text, boolean or empty cell as the data of ... TRIMMEAN (T-2256)").
  Note `TRIMMEAN("2.5","2.5")` is `#NUM!`, so the percent "2.5" reads as 2.5 (out of range), and `TRIMMEAN(..,Data!A1:B2)`
  is a 2x2 spill, one result per percent cell: Data!A1:B2 = {FALSE,"TRUE";"1",TRUE} gives {2.5,#VALUE!;#NUM!,#NUM!}
  (A253:B254; A439:B440 for TRUE). Only the anchors A253 and A439 fail today; the lift already matches.
- Non-numeric text stays `#VALUE!`, before the percent's own error: `TRIMMEAN("abc",0)`, `TRIMMEAN("",0)`,
  `TRIMMEAN("abc",#NAME?)` and `TRIMMEAN("",#VALUE!)` are `#VALUE!` (A259-A385; ours already matches these).
- Recorded in statistics.md "TRIMMEAN of a typed scalar (T-3781)". Code: trimmean.rs `fn_trimmean`.

## Acceptance criteria
- [ ] The 15 cells below pass.
- [ ] statistics.md entry's Ours line updated.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0159: the task is done when the gen-m9 cells below (T-3781's arg-pairs batch, excel-graph, 2026-11-01) pass and
their known-failure lines leave eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure line in any baseline.
- gen-m9/4fc6bdaec0e5583d Calc!A199, A205, A211, A217, A235, A241, A247, A253, A391, A397, A403, A421, A427, A433, A439

## Out of scope
- An error cell as the data against a text percent (T-3788).
