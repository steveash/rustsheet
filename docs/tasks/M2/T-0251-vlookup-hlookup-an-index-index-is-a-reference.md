---
id: T-0251
title: "VLOOKUP/HLOOKUP: an INDEX(...) index is a reference (text or error cell is #REF!, not found is #N/A)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
VLOOKUP/HLOOKUP treat an index given as `INDEX(range,n)` as a reference to the indexed cell, like
`OFFSET`/`INDIRECT` already are: a text or error cell is `#REF!` when the value is found and the
lookup is `#N/A` when it is not (today: `#VALUE!` / the cell's error, even when not found).

## Context
Evidence: probes/lookup-index-shapes (excel-graph, 2026-10-06, T-0250), cells H2, H3, H11, H12 (the
only failing cells of that case); rule in docs/excel-semantics/lookup.md "VLOOKUP / HLOOKUP search
before validating the index". `lookup_index_arg` (functions/lookup_and_reference/index_arg.rs) keeps
a reference only when `evaluate_node_with_reference` returns a `Range`; INDEX returns a value there.
Make INDEX's reference form reachable from `evaluate_node_with_reference` (check other callers that
already want INDEX as a reference before adding a special case).

## Acceptance criteria
- [ ] probes/lookup-index-shapes H2, H3, H11, H12 leave eval/baseline/probes.txt, no new F lines.
- [ ] Unit tests for the four forms.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions that take INDEX results.
