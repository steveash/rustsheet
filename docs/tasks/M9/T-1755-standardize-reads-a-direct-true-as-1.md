---
id: T-1755
title: "STANDARDIZE reads a direct TRUE as 1"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
STANDARDIZE reads a direct TRUE as 1 (`STANDARDIZE(TRUE,3,2)` is -1).

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q58: Excel -1, ours #VALUE!. Text "5" already matches.
An array element reads the same way: probes/stat-later-argument-lift L14 `ROUND(INDEX(STANDARDIZE({1.5;TRUE},1,2),2),12)`
is 0 in Excel, ours `#VALUE!` (T-1290; the kernel is `standardize_of`, statistical/standardize.rs).

## Acceptance criteria
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
