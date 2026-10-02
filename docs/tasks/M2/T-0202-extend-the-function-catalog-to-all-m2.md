---
id: T-0202
title: "Extend the function catalog to all M2 functions"
milestone: M2
lane: harness
status: in-progress
priority: P2
size: S
depends_on: []
---

## Goal
Add HLOOKUP, INDEX, MATCH, SUMIFS, COUNTIFS, ROUNDUP, ROUNDDOWN, NOT, LEFT, RIGHT, MID, TRIM, UPPER, LOWER, CONCAT, YEAR, MONTH, DAY to eval/catalog/functions.yaml with correct arg modes.

## Context
Schema in docs/architecture/synthetic-generator.md §3.

## Acceptance criteria
- [ ] Catalog validates (add a schema test in tools/tests); generator can emit each function (test).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
