---
id: T-2739
title: "Bracket the typed string-literal length limit: 256 characters are accepted, 7,996 refused"
milestone: M7
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
syntax.md records the longest string literal Excel takes in a typed formula, bracketed by an accepted and a refused
probe, and whether a stored (file) formula has the same limit.

## Context
T-2709 (excel-graph 2026-10-25) found a typed `="` + 256 letters + `"` accepted (probes/t2709-typed-literal-256) and one
of 7,996 letters refused (Graph PATCH 400 `InvalidArgument`), while the published figure is 255 (syntax.md "Formula
length", the T-1397 note that split its literals to 255 units for this reason). Bisect between 256 and 7,996 with typed
`entered` cells (each refused length in its own case: one 400 stops the fill), then try the found bound stored, and in a
function argument (`LEN("…")`). If the engine's parser or importer limits literals differently, file a task.

## Acceptance criteria
- [ ] Probes bracket the typed limit (largest accepted, smallest refused, dated) and the stored one.
- [ ] syntax.md records both; a task is filed for any engine mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Text produced by functions (REPT, CONCAT results), which has its own 32,767 limit.
