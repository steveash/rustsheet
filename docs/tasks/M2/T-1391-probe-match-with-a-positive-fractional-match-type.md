---
id: T-1391
title: "Probe MATCH with a positive fractional match_type (0 < m < 1): exact or ascending? (follow-up of T-1304)"
milestone: M2
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence (excel-probe skill) for `MATCH(x,{...},0.5)`: exact, or ascending by sign? Then make the engine match.

## Context
T-1304 floored match_type (fn_match, lookup_and_reference/mod.rs), which leaves 0 < m < 1 as exact (floor 0). No Excel cell shows what Excel does. Record in docs/excel-semantics/lookup.md.

## Acceptance criteria
- [ ] A probe case with oracle values for 0.5, 0.0001 and 0.999 over ascending data; the engine matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Oracle (excel-graph) probe `probes/match-fractional-match-type`: `MATCH(2.5,{1..5},m)` is 2 for m = 0.5, 0.0001 and 0.999 (ascending), `#N/A` for m = 0. The engine now takes only the sign of match_type; the earlier floor made 0 < m < 1 exact. Recorded in docs/excel-semantics/lookup.md.
