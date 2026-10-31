---
id: T-3093
title: "Probe the near-zero snap of LAMBDA calls in other positions (LET-bound, parameter, chained, under a picker, as an aggregate argument)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3083]
---

## Goal
docs/excel-semantics/numbers.md "Near-zero snapping" records, with excel-graph evidence, whether a LAMBDA body's root
`+`/`-` snaps in each call position below, and a task is filed for every position where the engine differs.

## Context
- T-3083 snaps the body of every LAMBDA called through `Model::call_lambda` that is not a defined name
  (`LambdaCaller::root_snap`, functions/logical/lambda.rs). Only `=LAMBDA(x,x+0.2-0.3)(0.1)` and
  `=LAMBDA(x,x+7*2^-52-1)(1)` are probed (probes/t3049-snap-under-pickers B17, C17).
- Unprobed positions where the engine now snaps: a LET-bound LAMBDA (`=LET(f,LAMBDA(x,x+0.2-0.3),f(0.1))`), a
  LAMBDA passed as a parameter (`=LAMBDA(g,g(0.1))(LAMBDA(x,x+0.2-0.3))`), a chained call, and a call under a
  picker (`=IF(TRUE,LAMBDA(x,x+0.2-0.3)(0.1))`).
- Unprobed positions where it does not snap: a call as an aggregate argument or under ISREF
  (`=SUM(LAMBDA(x,x+0.2-0.3)(0.1))`, which goes through `keeping_references`), a call under `+0`, a grouped
  body (`=LAMBDA(x,(x+0.2-0.3))(0.1)`), the LAMBDA helpers (MAP, REDUCE, BYROW) and a defined-name LAMBDA.
- Found by the T-3083 reviewers. Use each form at k = 7 (`1+7*2^-52-1`) and k = 8 too, as t3049 does.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) covers each position above at 0.1+0.2-0.3, k = 7 and k = 8, filled by excel-graph.
- [ ] numbers.md records the finding per position; an engine task is filed for each mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (the filed tasks do).
