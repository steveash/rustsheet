---
id: T-3082
title: "Probe TEXT of 3000000 under tagged literals-only calendar prefixes ([$-40C]B2, [$-409]B2)"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(3000000,"[$-40C]B2")` and `"[$-409]B2"` (a positive value past the maximum serial under a tagged literals-only calendar prefix) match Excel.

## Context
T-3050 made `format_number` (formatter/format.rs) print the literals for any literals-only date section past the maximum serial, which also exempts tagged prefixes. Only the untagged `b1`/`b2` forms are oracle-evidenced (probes/t3050-b1-large-positive); the tagged negative form is "-" (T-2388).

## Acceptance criteria
- [ ] Probe both tagged codes on 3000000 (excel-graph); record in number-formats.md; fix ours if it differs, baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
