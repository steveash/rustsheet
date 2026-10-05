---
id: T-0247
title: "IS* functions (ISERROR, ISNUMBER, ...) lift over array arguments and spill"
milestone: M2
lane: functions-logic
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`ISERROR` and the other IS* functions given an array or multi-cell range return an array of the same shape (and spill), as Excel does.

## Context
Found by T-0920: probes/text-number-words F12 `=ISERROR({"inf","2","1e400"}*1)` spills
TRUE, FALSE, TRUE in Excel (F12:H12, excel-graph 2026-10-05); we return a single FALSE. Check which
IS* functions already lift (functions/information/) and reuse the existing array-lifting helper
(search before writing one, CLAUDE.md directive 3); probe the others with the excel-probe skill.

## Acceptance criteria
- [ ] probes/text-number-words F12:H12 pass; IS* lifting evidenced per function.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
