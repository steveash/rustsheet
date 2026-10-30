---
id: T-3237
title: "Probe the number format of typed mixed fractions"
milestone: M9
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish and match the number format Excel gives typed `3 1/2`, `-$3 1/2`, `-3 1/2%` and `+ 3 1/2`.

## Context
T-3178 showed these typed forms are numbers, but `CELL("format")` is `#VALUE!` on excel-graph, so the format is UNVERIFIED; ours is General (`parse_typed_number`, parse_formatted.rs, `TODO(T-3237)`). Use the ENTERED-log numberFormat as in docs/research/non-glued-typed-input-2026-10-19.md, or a desktop oracle.

## Acceptance criteria
- [ ] An oracle-backed probe records the format of each typed form; ours matches or a fix lands.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
