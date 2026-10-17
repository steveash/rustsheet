---
id: T-1752
title: "SKEW.P of fewer than three numbers is #DIV/0!, not 0"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SKEW.P of a data set with fewer than three numbers is #DIV/0!, not 0.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q16 (`SKEW.P(A1:A2)`) and S!Q19 (`SKEW.P(C1:C5)`, two numbers after text/bool/blank are skipped): Excel gives #DIV/0!, ours 0. SKEW.P(A1) and SKEW.P(N1:N3) already match.

## Acceptance criteria
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
