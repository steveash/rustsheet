---
id: T-1914
title: "SUBTOTAL over TAKE of SWITCH/IFS with an array selector spills a 2x2 block of #VALUE!"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBTOTAL(9,TAKE(SWITCH({1,2},1,H1:H2,2,I1:I2),2)) and the IFS twin spill a 2x2 block of #VALUE! as Excel's do.

## Context
probes/subtotal-take-array-selector A30:B31 and A40:B41 (Excel a 2x2 block of #VALUE!, ours a single #VALUE! in A30/A40); the IF twin A20:B20 (1x2 #VALUE!) matches. Found by T-1909; is_read_reference in static_analysis/aggregate.rs (T-1882).

## Acceptance criteria
- [ ] The listed probes/subtotal-take-array-selector cells pass; the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
