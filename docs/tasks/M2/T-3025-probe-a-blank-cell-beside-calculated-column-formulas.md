---
id: T-3025
title: "Probe a blank cell beside calculated-column formulas (T-2983 follow-up)"
milestone: M2
lane: harness
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
references.md states whether a blank data cell beside cells of a real calculated column (filled by typing) blocks the fill when a formula is typed into another row.

## Context
T-2983 could not isolate this: stored formulas are never a calculated column (rule 8a). Design: C2 entered `=A2*2` (fills C2:C4), clear C3 if the adapter can type an empty entry (else add that support), then C4 entered `=A4*3`; compare t2957-constant-then-refill. The engine blocks the fill (calculated_columns.rs) but unprobed.

## Acceptance criteria
- [ ] Probe filled on excel-graph and baselined; references.md rule updated; an engine task is filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
