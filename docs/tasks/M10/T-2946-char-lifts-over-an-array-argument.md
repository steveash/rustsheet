---
id: T-2946
title: "CHAR lifts over an array argument"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`CHAR` over an array or range argument returns an array of characters, as Excel does, instead of `#N/IMPL!`:
`=LET(s,SEQUENCE(10),CONCAT(CHAR(48+s)))` is "123456789:".

## Context
- Found in T-2759: the probes/t2759-lunar-packed-2101 … -9101 cells (158) compute
  `CONCAT(IF(...,CHAR(80+g),CHAR(48+g)))` over an array; Excel returns the packed string, the engine
  `#N/IMPL!` because `fn_char` (functions/text/char_code.rs) reads one number with `get_number`.
  Once CHAR lifts, those cells should pass (the lunar calendar past 2100 is implemented; they are
  the evidence it was built from).
- CODE and UNICHAR beside it already lift through `lift_text`; reuse it. Per-element errors
  (0, 256, text, blank) should follow the scalar rules; check probes/text-argument-modes for an
  existing CHAR row before probing anew (excel-probe skill).

## Acceptance criteria
- [ ] probes/t2759-lunar-packed-* pass (all 158 cells), with no new known failures.
- [ ] A unit or eval case covers CHAR over an array with an out-of-range element.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
