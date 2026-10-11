---
id: T-1314
title: "Stored #BUSY!: ARRAYTOTEXT of the cell alone is the error, plus the T-1228 probe cells (after T-1222)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-1222]
---

## Goal
Every stored-`#BUSY!` cell of probes/error-stored-busy-behaviors passes once the error kind exists, including ARRAYTOTEXT of the cell alone being the error `#BUSY!`.

## Context
Evidence: syntax.md "Stored `#BUSY!` behaviors" (T-1228). The probe's 10 baseline F lines (C1-C5, C7, C9, C11, C15, E1) fail today. Most fail because the engine has no stored `#BUSY!` kind (T-1222 adds it), but two need their own rules: C11 (`SUM(A1,1/0)` is ERROR.TYPE 2, where the engine takes the leftmost error and, after T-1222, would give 0) and C15 (a `#BUSY!` criterion does not match the `#BUSY!` cell, unlike `#N/A`). SPEC-0003 assumed ARRAYTOTEXT gives the text; Excel gives the error for a single cell, as for stored `#N/A` (C17). Do this after T-1222; re-run the probe after T-1222 and fix what remains (expect ARRAYTOTEXT, C11 and C15).

## Acceptance criteria
- [ ] All cells of probes/error-stored-busy-behaviors pass; baseline updated.
- [ ] SPEC-0003 matches the ARRAYTOTEXT result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
