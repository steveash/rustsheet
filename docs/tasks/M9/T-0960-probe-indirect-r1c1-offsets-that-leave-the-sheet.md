---
id: T-0960
title: "Probe INDIRECT R1C1 offsets that leave the sheet: wrap or clamp"
milestone: M9
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether an INDIRECT R1C1 relative offset that leaves the sheet wraps modulo the sheet size or clamps, and record it.

## Context
T-0957 saw `INDIRECT("R[-20]C",FALSE)` in B15 return 0, not #REF!, but the wrapped cell was empty so wrap and clamp look the same (docs/excel-semantics/lookup.md, "INDIRECT R1C1 relative offsets"). Put a distinguishing value at the wrapped address (e.g. row 1048571 for R[-20]C from row 15, and row 1 for a positive overflow) and probe both directions and columns.

## Acceptance criteria
- [ ] Probe case with oracle sidecar for row and column overflow in both directions; the lookup.md entry updated to VERIFIED (or the engine mismatch filed).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `indirect-r1c1-offset-overflow` (excel-graph oracle): R1C1 relative offsets wrap modulo the sheet size in both directions for rows and columns; magnitude >= sheet size is #REF!. Engine already matches; lookup.md now VERIFIED. Eval: 7 new probe cells, all passing.
