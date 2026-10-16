---
id: T-1684
title: "ASC returns its text unchanged in en-US (full-width ASCII, ideographic space and punctuation are not narrowed)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ASC matches Excel in en-US: every input comes back unchanged.

## Context
Found by T-1626 (probes/t1626-asc, excel-graph): `ASC` of full-width letters, digits, ideographic space, `！`, `＂`, `＇`, `ａ`&`ｂ` returns the text unchanged, but the engine narrows them (9 failing cells: C5, C6, C11-C16, C24). See docs/excel-semantics/text.md ("ASC and JIS in en-US").

## Acceptance criteria
- [ ] probes/t1626-asc passes all cells; `make eval-baseline` shrinks the known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- JIS (a #NAME? on the web).
