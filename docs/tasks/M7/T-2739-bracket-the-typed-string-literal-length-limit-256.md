---
id: T-2739
title: "Bracket the typed string-literal length limit: 256 characters are accepted, 7,996 refused"
milestone: M7
lane: research
status: done
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
- [x] Probes bracket the typed limit (largest accepted, smallest refused, dated) and the stored one.
- [x] syntax.md records both; a task is filed for any engine mismatch.
- [x] `make check` green; reviewers approve.

## Out of scope
- Text produced by functions (REPT, CONCAT results), which has its own 32,767 limit.

## Outcome
A string literal holds at most 4,095 UTF-16 units (its value: a doubled quote counts once; 2,047 emoji pass, 2,048 fail), typed and stored, per literal
and in function arguments; 4,096 is refused (typed Graph 400, stored createSession 501). Bisected with scratch fills
(4,126, 4,110, 4,102, 4,098, 4,096 refused; 2,191 to 4,095 accepted); the accepted cases are probes/t2739-*. Recorded in
syntax.md "Formula length". The engine accepts longer literals: T-2762.
