---
id: T-1345
title: "Probe IMAGE sizing, height and width validation (sizing 9 is #VALUE!)"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for which IMAGE `sizing`, `height` and `width` values are `#VALUE!`, and the engine validates them the same way.

## Context
probes/rich-errors-shapes-urls A16:B16 (T-1189): `IMAGE({url,url},"alt",9)` is `#VALUE!` per element in Excel, `#BLOCKED!` in the engine, which does not validate `sizing`/`height`/`width` (functions/information/offline_errors.rs `offline_args`). Probe sizing 0..3, -1, 4, 1.5, text, sizing 3 with and without height/width, height/width 0 and negative, sizing 0 with height given, each scalar and over an array source, wrapped in ERROR.TYPE (raw IMAGE errors come back undecodable).

## Acceptance criteria
- [ ] A probes case covers those arguments with excel-graph values, recorded in information.md ("IMAGE and STOCKHISTORY never fetch").
- [ ] The engine matches them; rich-errors-shapes-urls A16:B16 leave the baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
