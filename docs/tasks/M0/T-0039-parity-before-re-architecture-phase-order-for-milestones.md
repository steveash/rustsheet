---
id: T-0039
title: "Parity before re-architecture: phase order for milestones"
milestone: M0
lane: research
status: in-progress
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
- [ ] Decision record: two phases (parity, then architecture and performance), the Phase 1 exit
  criteria, what is out of scope, and what stays allowed during Phase 1.
- [ ] docs/milestones.md states the phase order. M3, M4 and the performance half of M11 (now
  M13) are marked Phase 2. M11 keeps real-world fidelity in Phase 1.
- [ ] The open M3 tasks are `blocked` until Phase 1 exits.
- [ ] A task to catalog every Excel function with its parity scope (T-0040).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing routine prompts. The steward picks the critical lanes from docs/milestones.md.
- Renumbering milestones or moving task files.
