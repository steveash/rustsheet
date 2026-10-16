---
id: T-1665
title: "Probe AVEDEV sum of absolute deviations accumulation (T-1650 follow-up)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence on whether AVEDEV's sum of absolute deviations rounds each addition twice.

## Context
T-1650 showed AVEDEV's mean sum rounds twice, but its operand sets cannot discriminate the deviation sum (plain `+` and `extended_sum` agree in all 8 sets). Build sets whose |v - mean| form a discriminating triple (a, b, c with a 0 pad, mean exact), oracle-fill them in probes-numeric, then switch `fn_avedev` (functions/statistical/count_and_average.rs) if Excel double-rounds.

## Acceptance criteria
- [ ] Probe cells with oracle values; numbers.md updated; engine matches or a task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
