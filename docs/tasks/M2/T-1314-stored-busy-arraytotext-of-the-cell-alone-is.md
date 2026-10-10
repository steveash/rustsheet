---
id: T-1314
title: "Stored #BUSY!: ARRAYTOTEXT of the cell alone is the error, plus the T-1228 probe cells (after T-1222)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-1222]
---

## Goal
Every stored-`#BUSY!` cell of probes/error-stored-busy-behaviors passes once the error kind exists, including ARRAYTOTEXT of the cell alone being the error `#BUSY!`.

## Context
Evidence: syntax.md "Stored `#BUSY!` behaviors" (T-1228). The probe's 10 baseline F lines (C1-C5, C7, C9, C11, C15, E1) fail today because the engine has no stored `#BUSY!` kind; T-1222 adds it. SPEC-0003 assumed ARRAYTOTEXT gives the text; Excel gives the error for a single cell, as for stored `#N/A` (C17). Do this after T-1222; if T-1222 already makes all cells pass, only the ARRAYTOTEXT rule may remain.

## Acceptance criteria
- [ ] All cells of probes/error-stored-busy-behaviors pass; baseline updated.
- [ ] SPEC-0003 matches the ARRAYTOTEXT result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
