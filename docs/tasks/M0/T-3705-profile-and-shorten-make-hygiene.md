---
id: T-3705
title: "Profile and shorten make hygiene"
milestone: M0
lane: harness
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`make hygiene` (the critical path of every docs-only PR, and part of every heavy run) takes
noticeably less than its current 74-107 s, with every ratchet and check still run.

## Context
Owner request (2026-11-01, CI-speed investigation). Docs-only PR runs take p50 2.7 min, bounded by
the hygiene job (116 s in run 41288141328: checkout 23 s with full history, `make hygiene` 72 s,
deny 9 s); in heavy runs `make hygiene` took p50 104 s. It runs nine scripts plus the scripts'
unit tests; nobody has measured which of them dominates.

## Acceptance criteria
- [ ] Per-step times of `make hygiene` (locally and in one CI run) recorded in the Outcome.
- [ ] The slowest steps are made faster (caching parsed task/decision files, running independent
      checks in parallel, a shallower checkout where full history is not needed) without
      dropping any check; before/after CI times recorded.
- [ ] A decision record if the Makefile, scripts/ or .github/ change; `make check` green;
      reviewers approve.

## Out of scope
- Removing or weakening any ratchet.
