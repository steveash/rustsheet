---
id: T-2630
title: "CELL(contents) of a blank inside IF counts in COUNT and COUNTA and is #VALUE! in MODE.SNGL (T-2622 probe)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`COUNT(IF(TRUE,CELL("contents",C5)))` is 1 and `MODE.SNGL(IF(TRUE,CELL("contents",C5)))` is `#VALUE!` with C5 blank, as the written-out call is.

## Context
T-2622 implemented the written-out call only (`is_cell_contents_of_blank`, `statistical/count.rs`, `mode_functions.rs`) because a blank value of any other kind (a REDUCE result, gen-m9/d0854366830dcdee Calc!A40) is not counted. Excel gives 1 and `#VALUE!` through IF (oracle T-2622 probe, probes/t2622-contents-xmatch-take-lambda A6 and A18, baselined as known failures). Find what value CELL returns that IF passes on but REDUCE does not.

## Acceptance criteria
- [ ] The two IF forms match Excel; probe cells A6 and A18 leave the baseline; nothing regresses (baseline only shrinks).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
