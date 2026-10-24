---
id: T-2581
title: "TEXTSPLIT with a range row delimiter and an invalid match_mode, and TEXTBEFORE with a number delimiter from a reference and a negative instance (gen-m9, 3 cells)"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
TEXTSPLIT and TEXTBEFORE give Excel's results for these three calls.

## Context
From T-2436's probe of the untraced gen-m9 cells. text.md "TEXTSPLIT and TEXTBEFORE in the gen-m9 sweep (T-2436)". Probe first which part decides each: a row delimiter given as a range holding text (the 1x2 result means the column split wins or the row delimiters are read differently), a match_mode of 2 (`#VALUE!`; is ignore_empty 2.5 read first?), and a delimiter that does not occur, read from a one-cell range holding the number 1.005, with instance -2 (a number delimiter from a reference, or the negative instance with no match). Code: functions/text textsplit, textbefore.

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `TEXTSPLIT("a*","a",Data!A6:B12,FALSE,0)` is 1x2 {"","*"} (ours 2x1), `TEXTSPLIT(2,"a",Data!B5:D6,2.5,2,"a*")` with match_mode 2 is `#VALUE!` (ours "2"), and `TEXTBEFORE("42ABC",<one-cell range holding 1.005>,-2)` `#VALUE!` (ours `#N/A`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 3 known-failure lines in eval/baseline/gen-m9.txt, 3 formulas.
Examples:
- gen-m9/3e98b26cf6713e77 Calc!A383 `=FISHER((TEXTSPLIT("a*","a",Data!A6:B12,FALSE,0)^RECEIVED(38833,46585,439103,Data!$A11)))`: Excel #VALUE!, ours #VALUE!; inner `TEXTSPLIT("a*","a",Data!A6:B12,FALSE,0)` Excel "", ours ""
- gen-m9/8d3588d8df0812df Calc!A313 `=CHIINV(TEXTBEFORE((Data!D5&"ABC"),Data!E9:E9,-2),10)`: Excel #VALUE!, ours #N/A; inner `TEXTBEFORE((Data!D5&"ABC"),Data!E9:E9,-2)` Excel #VALUE!, ours #N/A
- gen-m9/984cba67bbe6972b Calc!A160 `=CHISQ.INV.RT(SWITCH(TEXTSPLIT(Data!D8,"a",Data!B5:D6,Data!A5,2,Data!E7),((-0.5)-0.2),IMDIV("3+4i","-i"),100),5)`: Excel #VALUE!, ours #NUM!; inner `TEXTSPLIT(Data!D8,"a",Data!B5:D6,Data!A5,2,Data!E7)` Excel #VALUE!, ours "2"
Cells:
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A383
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A313
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A160

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2436): these 3 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The rule's quirks are in docs/excel-semantics/ with the probe cells as evidence; new probe cells are gated or baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells of the same functions owned by other tasks.
