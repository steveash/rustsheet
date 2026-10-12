---
id: T-1391
title: "Probe MATCH with a positive fractional match_type (0 < m < 1): exact or ascending? (follow-up of T-1304)"
milestone: M2
lane: functions-lookup
status: in-progress
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
