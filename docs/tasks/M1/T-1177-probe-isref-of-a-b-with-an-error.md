---
id: T-1177
title: "Probe ISREF of a:b with an error or non-reference operand"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for `ISREF(a:b)` when an operand errors or is not a reference, with the engine matching it.

## Context
T-1166 made `is_reference` FALSE whenever `get_range` is not a range. Only the two-sheet form (probes/sheet-qualified-corner-order Calc!A7) has oracle evidence; see the UNVERIFIED note in docs/excel-semantics/references.md. Code: `is_reference` in crates/rustsheet-engine/src/functions/information/mod.rs.

## Acceptance criteria
- [ ] Probe `ISREF(A1:INDEX(A1:A3,5))`, `ISREF(A1:(A1 B2))` and `ISREF(A1:IF(TRUE,1))` with the excel-probe skill; engine matches or the mismatch is fixed; the references.md entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
