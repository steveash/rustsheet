---
id: T-1818
title: "Probe call-name spelling folds for dotless i, long s, micro, final sigma and ligatures"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's call-name spelling folds for ı/i, ſ/s, µ/μ, ς/σ and ligatures are probed and ours matches.

## Context
T-1762 keys `CallNameSpellings` on the upper-cased name; that also merges characters no probe covered (reviewer finding). Semantics: docs/excel-semantics/references.md "Unknown function call names".

## Acceptance criteria
- [ ] Probe cases (e.g. `=ſx(1)` then `=SX(1)`) with oracle values; the registry key matches them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
