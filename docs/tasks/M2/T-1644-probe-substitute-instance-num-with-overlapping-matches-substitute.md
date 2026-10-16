---
id: T-1644
title: "Probe SUBSTITUTE instance_num with overlapping matches (SUBSTITUTE(aaa,aa,x,2))"
milestone: M2
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's result for SUBSTITUTE with an instance_num when matches overlap is evidenced, and the engine matches it.

## Context
The engine counts overlapping instances (`text_sensitive`, `nth_overlapping_match` in functions/text/util.rs): `SUBSTITUTE("aaa","aa","x",2)` is "ax". No probe backs it (found in T-1586 review). Use the excel-probe skill; record the result in docs/excel-semantics/text.md.

## Acceptance criteria
- [ ] A probe case with oracle values covers overlapping instances (also with instance_num 1 and 3, and negative cases); the engine matches or the quirk is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
