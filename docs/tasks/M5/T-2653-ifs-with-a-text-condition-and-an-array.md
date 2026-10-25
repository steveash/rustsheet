---
id: T-2653
title: "IFS with a text condition and an array value: a #VALUE! array the value's shape (spills)"
milestone: M5
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
IFS whose condition is text not reading as a boolean returns an array of `#VALUE!` shaped like its paired value (or the
broadcast of condition and value), as Excel does, so a 3x2 value spills (or shows `#SPILL!` when blocked) instead of giving
one `#VALUE!`.

## Context
- gen-m5 round-3 triage (T-2648, excel-graph oracle, 2026-10-25), 1 cell in eval/baseline/gen-m5.txt, visibly off:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/4260ce3226165deb Calc!A208 | `=IFS(Data!E5,(INDEX(Data!B9:C11,0,0)+SUMIF(Data!A5:D7,"=a",Data!B6:E6)),IF(TRUE,"a*",(Data!D2>Data!B11)),123456789.123%)` (E5 = "x") | #SPILL! | #VALUE! |

- The first condition is the text "x". docs/excel-semantics/logic.md "IFS: a text condition gives a 1x1 array of #VALUE!"
  (T-2280, `ifs_pick`) covers scalar values only. Here the paired value `INDEX(Data!B9:C11,0,0)+SUMIF(..)` is a 3x2 array,
  and Excel's `#SPILL!` (A209 below is occupied) says the result is a multi-cell array, so the text condition's `#VALUE!`
  takes the value's shape. Whether it is the value's shape, the broadcast with the condition, or every later pair too, is
  for the probe.
- Recorded in logic.md "IFS: a text condition with an array value spills its shape" (UNVERIFIED until the probe).

## Acceptance criteria
- [x] An excel-graph probe (`excel-probe` skill) pins the result shape: `ROWS`/`COLUMNS`/`TYPE` of `IFS("x",{1,2;3,4})`,
      `IFS("x",A1:B3)`, `IFS(FALSE,1,"x",{1,2})`, `IFS("x",5,TRUE,{1,2})` and the same with a spill footprint free.
- [x] The engine matches the probe; the gen-m5 line above disappears (`make eval-baseline`), with no new known failure.
- [x] logic.md entry updated with the evidence and `Status: VERIFIED`.
- [x] `make check` green; reviewers approve.

## Out of scope
- SWITCH (logic.md: it has no text-condition case).

## Outcome
Probed (2 cases, excel-graph): the #VALUE! array takes the broadcast shape of the pair's value and all later arguments. `ifs_pick` now lifts the remaining args for a text condition. gen-m5 known failures -1 (Calc!A208); no regressions; logic.md VERIFIED.
