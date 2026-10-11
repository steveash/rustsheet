---
id: T-1336
title: "Probe near-zero snapping for AGGREGATE(9), SUBTOTAL(9/109) and PERCENTOF"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for near-zero snapping in `=AGGREGATE(9,4,A1:A3)`, `=SUBTOTAL(9,A1:A3)` and `=PERCENTOF(...)` over 0.1, 0.2, -0.3, and the engine matching it.

## Context
T-1131 showed a SUM call snaps its final addition anywhere (docs/excel-semantics/numbers.md "SUM forms"). AGGREGATE(9), SUBTOTAL(9/109) and PERCENTOF use `fn_sum_unsnapped` (sum.rs), unprobed. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cells with oracle values; numbers.md updated; `fn_sum_unsnapped` removed or kept with evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
