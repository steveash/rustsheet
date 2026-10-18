---
id: T-1698
title: "Probe XMATCH/XLOOKUP reference-form lookup arrays and thorn case"
milestone: M2
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
XMATCH and XLOOKUP agree with Excel on every reference form of lookup_array/return_array, and on the capital Thorn.

## Context
T-1669 switched XMATCH `lookup_array` and XLOOKUP `lookup_array`/`return_array` to `evaluate_node_with_reference` (single-cell references are one-cell vectors). That also changes `@`-intersected ranges, LET-bound references, single-cell defined names and 3-D spans (now `#VALUE!` rather than `#REF!`), none probed. Probe (excel-probe skill): `XMATCH(x,Sheet1:Sheet3!A1)`, `XMATCH(x,@A1:A3)`, `LET(r,A1,XMATCH(x,r))`, a single-cell defined name as lookup_array, XLOOKUP with a single-cell return_array holding a number or a blank, and `"Þ"="TH"`, `"Þ"<"TI"`. Fix the engine where it differs.

## Acceptance criteria
- [ ] A probe case records Excel's results for those cells and the engine matches; findings in docs/excel-semantics/lookup.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes `xmatch-xlookup-reference-forms` (M2) and `xmatch-xlookup-3d-lookup-array` (M8) with excel-graph oracle values; the engine already matched all 20 cells, so no engine change. Findings in docs/excel-semantics/lookup.md. Eval delta: +20 passing probe cells, no regressions.
