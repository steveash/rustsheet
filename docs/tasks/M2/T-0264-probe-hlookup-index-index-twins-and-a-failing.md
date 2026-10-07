---
id: T-0264
title: "Probe HLOOKUP INDEX(...) index twins and a failing INDEX index (INDEX(E1:E4,9))"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior is recorded for HLOOKUP with an INDEX(...) index and for an INDEX index that itself fails.

## Context
Follow-up of T-0251 (review). probes/lookup-index-shapes probes only VLOOKUP H2, H3, H11, H12; the HLOOKUP twins (INDEX(E1:E2,1) text, INDEX(...,2) error, found and not found) are implemented from the shared `lookup_index_arg` but unconfirmed. Also unprobed: `INDEX(E1:E4,9)` (#REF! from INDEX itself) when the lookup value is not found; the engine fails before the search.

## Acceptance criteria
- [ ] Probe cases added and oracle-filled; docs/excel-semantics/lookup.md updated.
- [ ] The engine matches, or the gap is baselined.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
