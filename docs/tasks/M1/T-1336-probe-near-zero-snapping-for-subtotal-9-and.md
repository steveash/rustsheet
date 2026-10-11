---
id: T-1336
title: "Probe near-zero snapping for AGGREGATE(9), SUBTOTAL(9/109) and PERCENTOF"
milestone: M1
lane: research
status: done
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
- [x] Probe cells with oracle values; numbers.md updated; `fn_sum_unsnapped` removed or kept with evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/near-zero-aggregate-forms` (8 cells, excel-graph): AGGREGATE(9) and SUBTOTAL(9/109) snap like SUM; the engine now matches (`subtotal_sum`, AGGREGATE dispatch). PERCENTOF is `#NAME?` on excel-graph, so it stays unsnapped and UNVERIFIED (desktop Excel, M14); the probe has no PERCENTOF cell. `fn_sum_unsnapped` now serves PERCENTOF only.
