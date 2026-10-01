---
id: T-0902
title: "CHAR text mismatches"
milestone: M9
lane: functions
status: todo
priority: P2
size: S
depends_on: []
---

## Goal
Fix the 29 CHAR cells that differ from Excel (likely code-page 1252 mapping for 128–159).

## Context
`make eval-show CASE=ironcalc-calc/TEXT/...` (find via report).

## Acceptance criteria
- [ ] CHAR known failures → 0; mapping table documented.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
