---
id: T-1665
title: "Probe AVEDEV sum of absolute deviations accumulation (T-1650 follow-up)"
milestone: M2
lane: research
status: done
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

## Outcome
Probe `double-rounding-avedev-deviations` (8 symmetric operand sets, 56 cells, excel-graph): AVEDEV's sum of absolute deviations rounds twice per addition. `fn_avedev` now accumulates the deviations with `extended_sum` (all 8 AVEDEV cells were one or two ulps off, now 0). The SUMPRODUCT control cells show the engine's SUMPRODUCT misses the same way; filed as T-1676.
