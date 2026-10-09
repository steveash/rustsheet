---
id: T-1239
title: "Commit a seeded gen-m6 suite with oracle values and file its gaps"
milestone: M6
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A committed, seeded `gen-m6` suite with excel-graph values scores the M6 date and time functions, and its gaps are filed as tasks.

## Context
Like T-1192 for gen-m5. T-1090's uncommitted sample (`rustsheet-gen --milestone M6 --count 25 --seed 13`) was all
accepted by excel-graph and the engine passed 957/1000 cells. Date and time misses seen there, to triage:
`YEARFRAC(11194,<text or blank>)` (Excel -31.65 in context, ours `#NUM!`), `DATEDIF(<cell>,36657,"MD")` (ours `#NUM!`),
`DATE(<cell>,NETWORKDAYS(<cell>,9857),<x>)` (ours `#NUM!`), `SUM(WORKDAY(MROUND(..),..,COUNTIF(..)))` (Excel 0, ours `#NUM!`),
`TIME(<cell>,8,34)` and `WEEKNUM(27631,NETWORKDAYS.INTL(..,6))` (Excel `#NUM!`, ours `#VALUE!`). Many other misses are M5
gaps already filed (FILTERXML, PHONETIC, BAHTTEXT, DBCS, PERCENTOF, MULTINOMIAL). Decide whether to commit after T-1238
lands (so the suite also covers lookup) or now for the date half.

## Acceptance criteria
- [ ] eval/suites/gen-m6 is committed with oracle sidecars (non-gating, like gen-m5), and each distinct engine gap is a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
