---
id: T-1396
title: "Probe MATCH descending forward continuation when nothing of the key type lies at or below the midpoint"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with oracle probes the exact rule for a descending MATCH when the backward skip finds no cell of the key's type between the live range's low bound and the midpoint.

## Context
T-1312 continues the search forward from the midpoint in that case, evidenced by one cell only (gen-m2 b4392a6514a36ade Calc!A277, `MATCH(-14,{FALSE,0.5},-1)` = 2). Probe more shapes (several other-typed cells at or below the midpoint with a hit far above it, text keys, longer vectors). Code: `SearchVector::approximate_search` in crates/rustsheet-engine/src/functions/lookup_and_reference/lookup_table.rs; entry: docs/excel-semantics/lookup.md "MATCH descending search skips backward".

## Acceptance criteria
- [ ] Probe cases with oracle values are in eval/suites/probes and the lookup.md entry states the verified rule (use the `excel-probe` skill); any engine mismatch is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/match-descending-forward-continuation (20 cells, oracle excel-graph). Engine already matches all 20; rule recorded in docs/excel-semantics/lookup.md. Eval delta: probes +20 cells passing, 0 regressions; no follow-ups.
