---
id: T-0248
title: "Probe ISEVEN/ISODD over numeric and mixed ranges; extend IS* lift probe coverage"
milestone: M2
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ISEVEN/ISODD over ranges match Excel, with durable oracle evidence, and the IS* lift probe covers its whole spill.

## Context
T-0247 saw a single `#VALUE!` (no spill) for ISEVEN/ISODD over a mixed range (docs/excel-semantics/information.md, UNVERIFIED). Also: `probes/is-functions-array-lift` checks only part of the 3x2 ISBLANK block (B12, A13, B13 unchecked), and no case uses a reference-returning argument (`ISBLANK(OFFSET(Data!A5,0,0))`, `ISNUMBER(INDEX(Data!A1:A6,1))`).

## Acceptance criteria
- [ ] Probe ISEVEN/ISODD over a numbers-only and a mixed range (oracle sidecar kept); semantics entry updated to VERIFIED and implemented if lifting.
- [ ] Extend the IS* probe check list and reference-returning arguments; re-baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
ISEVEN/ISODD over a range return one `#VALUE!` (no spill) for numbers-only and mixed ranges (excel-graph); the engine already matched, so semantics entry is now VERIFIED. IS* lift probe check list extended (B12, A13, B13, ISEVEN/ISODD block, OFFSET/INDEX reference arguments); all new cells pass. Baseline: probes case coverage 57 -> 68 checked cells, no fixes or regressions.
