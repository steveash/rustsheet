---
id: T-2782
title: "SPEC-0023 step 2: normalise a whitespace run to its line feeds and last spaces, wrapping the space count at 256"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-2781]
---

## Goal
A typed whitespace run prints as Excel stores it: only its line feeds and the spaces after the last
one, each count wrapping at 256 (300 spaces print as 44).

## Context
SPEC-0023 step 2 ("A run is a line-feed count plus a space count"). Evidence:
probes/t2776-formula-text-whitespace-shapes C19:C22, F21 (excel-graph, 2026-10-26). Exactly 256 spaces is
unprobed: probe it first (excel-probe skill) before relying on the byte reading.

## Acceptance criteria
- [ ] t2776-formula-text-whitespace-shapes C19:C22, F21 pass, plus any t2769 line-feed cell T-2781 handed over.
- [ ] A probe of 255, 256 and 257 typed spaces, and the parser matches it; no panic on any count.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Stored-text runs (T-2784).
