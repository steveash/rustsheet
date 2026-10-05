---
id: T-0107
title: "Number→text conversion with Excel's 15-significant-digit rule"
milestone: M1
lane: engine
status: in-progress
priority: P1
size: S
tier: opus
depends_on: [T-0009]
---

## Goal
`=CONCAT(0.1+0.2)` and `&` concatenation of numbers must produce Excel's text.

## Context
One shared number-to-text routine must be used by every coercion site (concatenation, text functions, criteria).

gen-m1 (T-0109 triage, oracle excel-graph 2026-10-05): 34 `&` cells, all in this routine
(`gen-m1/f91603cd8616521f` 33 cells, `gen-m1/57fe331cb7611dbf` A235):
- 21 cells need 15 significant digits or E notation: `0.1+0.2&100` (A27) Excel "0.3100", engine
  "0.30000000000000004100"; `1/3&7` (A21) Excel "0.3333333333333337"; `7+Data!$E5&10` (A188,
  E5 = 1E-15) Excel "710", engine "7.00000000000000110"; `1E+20&0.1` (A50) Excel "1E+200.1", engine
  "1000000000000000000000.1". So 1E+20 already switches to E notation, which narrows the bound in
  docs/excel-semantics/numbers.md. Cells: A21 A27 A29 A44 A50 A69 A73 A90 A120 A126 A140 A153 A186
  A187 A188 A196 A202 A213 A224 A225 A240.
- 13 cells print negative zero: `-0&0.5` (A20) Excel "00.5", engine "-00.5"; `-Data!C5&1` (A13, C5
  empty) Excel "01"; `(0.1&-'Q''1'!B$4)` (57fe… A235) Excel "0.10", engine "0.1-0". Cells: A7 A13 A20
  A28 A33 A71 A110 A112 A125 A177 A249 A250 and 57fe… A235. Semantics: numbers.md "Negative zero
  prints as 0".

## Acceptance criteria
- [ ] gen-m2 number-to-text cells leave eval/baseline/gen-m2.txt (T-0203 triage): 78bc88199fc96650 A154; 8eb18149d4245e1a A102; a339ea84ef64382e A32; b4392a6514a36ade A336 (`7+1E15` → "1000000000000010"); d0c29dd5c530e28c A34 (Calc sheet).
- [ ] The 34 gen-m1 cells above disappear from eval/baseline/gen-m1.txt, with no new F lines in any suite.
- [ ] probes A9 passes; all coercion sites use one routine (reviewer-quality checks).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
