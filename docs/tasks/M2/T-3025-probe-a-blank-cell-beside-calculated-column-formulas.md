---
id: T-3025
title: "Probe a blank cell beside calculated-column formulas (T-2983 follow-up)"
milestone: M2
lane: harness
status: done
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
- [x] Probe filled on excel-graph and baselined; references.md rule updated; an engine task is filed if ours differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/t3025-blank-beside-calculated` filled on excel-graph (2026-10-31): with `=[@Col]*2` typed into C2
(fills C2:C4), C3 cleared and `=[@Col]*3` typed into C4, only C4 changes (C2 2, C3 blank, C4 12). A blank beside a
real calculated column blocks the re-fill, like a constant. The engine already matches (5/5); no engine task needed.
Recorded as references.md rule 8(e).
