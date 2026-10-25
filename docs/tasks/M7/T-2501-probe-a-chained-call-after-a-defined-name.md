---
id: T-2501
title: "Probe a chained call after a defined-name LAMBDA: =MYFN(1)(2)"
milestone: M7
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's answer for a call chained after a call of a defined-name LAMBDA (`MYFN` defined as
`=LAMBDA(x,LAMBDA(y,x*y))`, `=MYFN(3)(4)`, and `MYFN` = `=LAMBDA(x,x)`, `=MYFN(1)(2)`) is a
committed probe, and the engine matches it.

## Context
T-2038 made the parser chain any name call; T-2405 probed the built-ins (references.md
"Calling a call's result") but left this form unprobed. Write one single-formula case per
formula (a refused formula loses its workbook), with the name in the spec's `names`, fill it with
`rustsheet-oracle fill -b excel-graph`, and record the result.

## Acceptance criteria
- [x] The probes are committed with oracle sidecars (or recorded as refused) and references.md
      "Calling a call's result" states the result; the engine matches or a follow-up is filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Two single-formula probes (excel-graph, 2026-10-25): probes/chained-calls-name-curried
  (`MYFN` = `=LAMBDA(x,LAMBDA(y,x*y))`, `=MYFN(3)(4)` = 12) and probes/chained-calls-name-value
  (`MYFN` = `=LAMBDA(x,x)`, `=MYFN(1)(2)` = `#NAME?`, the returned number called as a LET-bound
  value is). Neither workbook was refused. The engine already matches both (2 new passing cases,
  no code change). Recorded in references.md "Calling a call's result".
