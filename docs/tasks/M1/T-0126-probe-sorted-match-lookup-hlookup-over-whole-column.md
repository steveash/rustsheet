---
id: T-0126
title: "Probe sorted MATCH/LOOKUP/HLOOKUP over whole column/row with blank tail"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for approximate-match lookups (MATCH type 1/-1, LOOKUP, HLOOKUP/VLOOKUP sorted) over whole-column/row references with a blank tail, and the engine matches it.

## Context
T-0119 bounds search vectors at the last used row/column, so `MATCH(99,A:A,1)` now returns the last used row (3) instead of 1048576, and `HLOOKUP(99,1:2,2)` the last column's value. Pinned by unit tests in test_fn_match.rs / test_fn_hlookup.rs without oracle evidence. See docs/excel-semantics/lookup.md. Also verify `LOOKUP(30,A:A,E:E)` (result vector past the used area; currently 0).

## Acceptance criteria
- [ ] Probe cases in eval/suites/probes with oracle values; the unit tests and docs/excel-semantics/lookup.md match them (fix the engine if they differ).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
