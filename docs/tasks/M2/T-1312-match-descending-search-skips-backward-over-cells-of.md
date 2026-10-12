---
id: T-1312
title: "MATCH descending search skips backward over cells of another type (gen-m2 f7e6d351c8538ef5 A334)"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A descending MATCH (match_type < 0) steps backward past cells of another type, as Excel does.

## Context
Rule and evidence: docs/excel-semantics/lookup.md "MATCH descending search skips backward" (probes/match-descending-argcheck J1-J8). Engine: `SearchVector::approximate_search` in crates/rustsheet-engine/src/functions/lookup_and_reference/lookup_table.rs skips forward today. Fixes gen-m2/f7e6d351c8538ef5 Calc!A334.

## Acceptance criteria
- [x] probes/match-descending-argcheck J1-J8 and gen-m2 f7e6d351c8538ef5 A334 pass; baseline updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`SearchVector::approximate_search` now steps backward (down to the live range's low bound) for a descending MATCH, and falls back to the forward scan when nothing of the key's type lies at or below the midpoint (needed by gen-m2 b4392a6514a36ade A277: `MATCH(-14,{FALSE,0.5},-1)` = 2). Eval delta: gen-m2 known failures 29 -> 28 (f7e6d351c8538ef5 A334 fixed), probes 2849 -> 2847 (J1-J8 area), no regressions.
