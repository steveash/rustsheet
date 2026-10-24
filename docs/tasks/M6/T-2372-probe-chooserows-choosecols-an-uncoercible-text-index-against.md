---
id: T-2372
title: "Probe CHOOSEROWS/CHOOSECOLS: an uncoercible text index against a later error argument, and which of two errors wins"
milestone: M6
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result is known for a CHOOSEROWS/CHOOSECOLS index list that holds uncoercible text (`"x"`) before a later error argument, and for two different error arguments.

## Context
Filed by T-2273 (reviews). Ours returns the `#VALUE!` of an uncoercible text index at once, but lets an error in a later argument beat an out-of-range index (lookup.md "CHOOSEROWS and CHOOSECOLS: an error argument beats an out-of-range index"). Which error wins between two errors is also unprobed; ours keeps the first in argument order. Write one probe under eval/suites/probes (tag M6), fill it on excel-graph, record the answers in lookup.md, and file an engine/functions task for any cell where ours differs.

## Acceptance criteria
- [ ] A probe with `CHOOSEROWS(A1:B2,"x",NA())`, `CHOOSECOLS(A1:B2,"x",1/0)`, `CHOOSEROWS(A1:B2,NA(),1/0)` and `CHOOSEROWS(A1:B2,1/0,NA())` is filled on excel-graph.
- [ ] lookup.md states the answers; any cell where ours differs has its own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing a change (a separate task if ours differs).
