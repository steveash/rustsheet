---
id: T-1704
title: "FILTERXML with an array xml argument: Excel for the web gives #VALUE! per element (probes/filterxml-evidence-floor D28)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FILTERXML with an array xml argument matches Excel for the web.

## Context
probes/filterxml-evidence-floor Sheet1!D28 (`FILTERXML({...;...},"//a")`) fails in eval/baseline/probes.txt; Excel gives #VALUE! per element. docs/excel-semantics/web.md.

## Acceptance criteria
- [ ] probes/filterxml-evidence-floor D28 passes and is removed from the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
