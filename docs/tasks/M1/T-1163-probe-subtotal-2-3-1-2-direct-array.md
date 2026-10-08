---
id: T-1163
title: "Probe SUBTOTAL(2/3,{1,2}) direct array and lambda arguments (follow-up of T-1154)"
milestone: M1
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBTOTAL 2, 3, 102 and 103 with a direct array or lambda argument match Excel, with oracle evidence.

## Context
T-1154 moved COUNT/COUNTA onto `subtotal_walk_areas`; `subtotal_count_with` (functions/subtotal.rs) returns `#N/IMPL` for a direct array or lambda argument, where COUNT used to ignore it (0) and COUNTA already returned `#N/IMPL`. Neither is evidenced; Excel likely gives `#VALUE!` for a non-reference SUBTOTAL argument.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers `SUBTOTAL(2,{1,2})`, `SUBTOTAL(3,{1,2})`, 102/103 and a lambda argument; the finding is in docs/excel-semantics/references.md.
- [ ] The engine matches the oracle; eval baseline shrinks accordingly.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
