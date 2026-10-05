---
id: T-0039
title: "Parity before re-architecture: phase order for milestones"
milestone: M0
lane: research
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Agents work toward near-100% Microsoft 365 Excel parity, backed by a comprehensive oracle test
suite, before starting the dependency-graph, storage and performance re-architecture.

## Context
2026-10-05, owner request: Excel parity and completeness take precedence over the performance
and scale milestones. A re-architecture is easier and safer once the test suite is the best it
can be. Functions that talk to external systems (COM, web services, …) are out of scope.
Today the milestones are worked in numeric order, so M3 (dependency graph) and M4 (storage)
come before most function families, and the `perf` lane is on the critical-lane line.

## Acceptance criteria
- [x] Decision record: two phases (parity, then architecture and performance), the Phase 1 exit
  criteria, what is out of scope, and what stays allowed during Phase 1.
- [x] docs/milestones.md states the phase order. M3, M4 and the performance half of M11 (now
  M13) are marked Phase 2. M11 keeps real-world fidelity in Phase 1.
- [x] The open M3 tasks are `blocked` until Phase 1 exits (T-0403, already in flight in its own
  PR, is left to finish).
- [x] A task to catalog every Excel function with its parity scope (T-0040).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing routine prompts. The steward picks the critical lanes from docs/milestones.md.
- Renumbering milestones or moving task files.

## Outcome
2026-10-05: decision 0027 sets the phase order. Phase 1 is parity (M0, M1, M2, M5–M11), Phase
2 is architecture and performance (M3, M4, and the new M13 split out of M11), Phase 3 is the
release (M12). docs/milestones.md holds the Phase 1 exit criteria and the `**Phase 1:**
exited` marker. T-0400..T-0402 are blocked until then. T-0403, a small local perf fix already in
its own PR, was left to finish. T-0040 is filed for the full function catalog.
