---
id: T-1263
title: "Probe near-zero snapping for SUBTOTAL(9/109), AGGREGATE(9), PERCENTOF and the cell order of a 2-D range in SUM (follow-up of T-1131)"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for near-zero snapping of SUBTOTAL(9/109), AGGREGATE(9) and PERCENTOF, and for the cell order SUM uses in a 2-D range; the engine matches it.

## Context
T-1131 found that every SUM call snaps (numbers.md "Near-zero snapping"). Unprobed: the three callers that use `fn_sum_unsnapped` (AGGREGATE(9) in functions/aggregate.rs, PERCENTOF in mathematical.rs) or `subtotal_sum` (functions/subtotal.rs); and the 2-D order: every 2x2 layout in probes/near-zero-snap-sum-forms snaps in either row-major or column-major order. Use a 2x2 where only one order ends on opposite-sign near-equal addends.

## Acceptance criteria
- [ ] Probe cells with oracle values for each form; numbers.md records them; the engine matches (baseline shrinks or a DEVIATION entry).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
