---
id: T-1329
title: "TEXT of a tiny negative under a [s]-only format is '-0', not '-1'"
milestone: M1
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1268]
---

## Goal
`TEXT(-1E-10,"[s]")` is "-0" in both date systems, as Excel gives.

## Context
probes/date-system-1900-negatives A18 and probes/date-system-1904-negatives A18 (excel-graph): Excel keeps the minus on a value that rounds to 0 seconds; the engine gives "-1" (it rounds the negative away from zero, or floors it). docs/excel-semantics/dates.md "The 1904 date system". Code: the elapsed-seconds path of formatter/format.rs. Check `TEXT(-1.5,"[s]")` ("-129600", A17) stays passing; related negatives work is T-0934.

## Acceptance criteria
- [ ] A18 passes in both cases and leaves the baseline; no other eval cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
