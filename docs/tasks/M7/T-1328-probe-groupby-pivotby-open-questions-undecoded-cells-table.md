---
id: T-1328
title: "Probe GROUPBY/PIVOTBY open questions: undecoded cells, table relationship, header detection (SPEC-0006 step 7)"
milestone: M7
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-1276]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
Every open GROUPBY/PIVOTBY question listed in SPEC-0006 Rollout step 7 has oracle evidence or is moved to M14 Phase 4.

## Context
Questions: values shorter than keys (G!AB193) and total_depth 3 with one key (G!J217), both undecoded by excel-graph; field_relationship 1 with total_depth 2 (G!J169, no value returned); PIVOTBY with two value columns gives `""` Totals (P!Y97); the multi-key sort array semantics (G!AB145); the field_headers auto-detect rule; errors as keys; number key equality (0.1+0.2 vs 0.3, -0 vs 0, a numeric 0 key vs the blank key); all-FALSE filter `#VALUE!` vs `#CALC!` (G!A241). Use the excel-probe skill; decision 0039 for anything only desktop Excel can answer.

## Acceptance criteria
- [ ] Findings recorded in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY" and SPEC-0006 updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
