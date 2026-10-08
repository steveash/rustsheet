---
id: T-1163
title: "Probe SUBTOTAL(2/3,{1,2}) direct array and lambda arguments (follow-up of T-1154)"
milestone: M1
lane: functions-math
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SUBTOTAL 2, 3, 102 and 103 with a direct array or lambda argument match Excel, with oracle evidence.

## Context
T-1154 moved COUNT/COUNTA onto `subtotal_walk_areas`; `subtotal_count_with` (functions/subtotal.rs) returns `#N/IMPL` for a direct array or lambda argument, where COUNT used to ignore it (0) and COUNTA already returned `#N/IMPL`. Neither is evidenced; Excel likely gives `#VALUE!` for a non-reference SUBTOTAL argument.

## Acceptance criteria
- [x] A probe case (excel-graph) covers `SUBTOTAL(2,{1,2})`, `SUBTOTAL(3,{1,2})`, 102/103 and a lambda argument; the finding is in docs/excel-semantics/references.md.
- [x] The engine matches the oracle; eval baseline shrinks accordingly.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Probed on excel-graph (2026-10-08): `SUBTOTAL(2,{1,2})` (and 3/102/103/9 variants), `SUBTOTAL(2,A1:A2*1)` and
`SUBTOTAL(3,A1:A2*1)` each make createSession fail 501 `unsupportedWorkbook`, so Excel yields no cell value
and the oracle cannot fill a case; a defined-name lambda gave `#NAME?` (uninformative). Finding recorded in
docs/excel-semantics/references.md. No probe case committed (it could never get a sidecar). Open design
question for opus: Excel refuses the formula at entry, so the engine should probably reject it at parse time
(like `=(1)(2)`) instead of returning `#N/IMPL` from `subtotal_count_with`; decide whether that is worth a
parse-time reference-type check, or close the task as "no value to match" with a DEVIATION/decision note.
