---
id: T-1753
title: "HARMEAN of two empty ranges is #N/A, not #DIV/0!"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
HARMEAN of arguments with no numbers (two empty ranges) is #N/A, not #DIV/0!.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q50 (`HARMEAN(D3:D4)`): Excel #N/A, ours #DIV/0!.

## Acceptance criteria
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
