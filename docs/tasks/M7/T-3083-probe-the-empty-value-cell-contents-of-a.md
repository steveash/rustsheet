---
id: T-3083
title: "Probe the empty value (CELL contents of a blank, IFERROR of a blank) in DROP, CHOOSEROWS, WRAPCOLS, COLUMNS, SCAN, REDUCE, BYROW, BYCOL and VSTACK(,1)"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for how the array-taking functions T-3053 did not probe treat the empty value, and an
information.md entry recording it.

## Context
- T-3053 made the empty value (`CalcResult::EmptyArg`: `CELL("contents",<blank>)`, `IFERROR(<blank>,5)`, `IFNA(..)`)
  `#VALUE!` as the array argument of VSTACK, HSTACK, TOCOL, TOROW, TAKE, CHOOSECOLS, FILTER and MAP (`reject_empty_value`
  in crates/rustsheet-engine/src/functions/spill_functions.rs), each probed in probes/t3030-cell-contents-blank-shapers,
  t2466-cell-contents-stack and t3053-blank-reference-wrappers (excel-graph). SORT, UNIQUE and EXPAND accept it as 0.
- Unprobed, and read as 0 by ours: DROP, CHOOSEROWS, WRAPCOLS, COLUMNS, TRANSPOSE, SCAN, REDUCE, BYROW, BYCOL, and an
  omitted argument in a stack (`VSTACK(,1)`, `TOCOL(,)`).
- Write one probe case (excel-probe skill) with `DROP(CELL("contents",Data!C11),0)`, `CHOOSEROWS(..,1)`,
  `WRAPCOLS(..,1)`, `COLUMNS(..)`, `TRANSPOSE(..)`, `SCAN(0,..,LAMBDA(a,v,a+v))`, `REDUCE(0,..,LAMBDA(a,v,a+v))`,
  `BYROW(..,LAMBDA(r,1))`, `BYCOL(..,LAMBDA(c,1))`, the same with `IFERROR(Data!C11,5)` in place of CELL, and
  `VSTACK(,1)`, `HSTACK(,1)`, `TOCOL(,)`, with Data!C11 blank.

## Acceptance criteria
- [ ] The probe case is oracle-filled; its known failures are in eval/baseline/probes.txt.
- [ ] information.md, the "Ours (since T-3053)" paragraph of "CELL("contents", ref) as a VSTACK argument", records the
      results; a follow-up engine task is filed for every function that differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the results (the follow-up task).
