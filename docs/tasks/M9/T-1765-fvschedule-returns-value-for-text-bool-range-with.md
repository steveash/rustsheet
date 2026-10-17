---
id: T-1765
title: "FVSCHEDULE returns #VALUE! for text/bool/range-with-text/blank schedule elements and text principal (probe fin-fvschedule)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FVSCHEDULE gives Excel's #VALUE! for non-numeric schedule elements.

## Context
Probe `probes/fin-fvschedule` A3, A4, A13, A14, A16, A18, A19: text, boolean or text-cell elements in the schedule, a text-only schedule, a TRUE principal and an empty-string schedule are #VALUE! in Excel; we skip them or coerce. See docs/excel-semantics/financial.md (T-1625 entry).

## Acceptance criteria
- [ ] all failing cells of probes/fin-fvschedule pass; baseline shrinks
- [ ] `make check` green; reviewers approve.

## Out of scope
-
