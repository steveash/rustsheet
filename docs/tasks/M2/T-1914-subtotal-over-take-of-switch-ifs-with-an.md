---
id: T-1914
title: "SUBTOTAL over TAKE of SWITCH/IFS with an array selector is #SPILL!, not #VALUE!"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBTOTAL(9,TAKE(SWITCH(array selector,...),2)) and the IFS twin give #SPILL! as Excel's do.

## Context
probes/subtotal-take-array-selector A11, A12 (Excel #SPILL!, ours #VALUE!); the IF twin A10 (#VALUE!) matches. Found by T-1909; is_read_reference in static_analysis/aggregate.rs (T-1882).

## Acceptance criteria
- [ ] The listed probes/subtotal-take-array-selector cells pass; the baseline shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
