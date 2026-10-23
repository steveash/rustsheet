---
id: T-2533
title: "IM* text writes 1i when a part is not exactly 1: COMPLEX, IMDIV and IMPOWER (gen-m9, 10 cells)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
IM* text writes the coefficient 1 ("1i", "-1i") whenever the imaginary part is not exactly ±1, as Excel does, and omits it only for exactly ±1.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"). `COMPLEX(0.1,1+2^-52,"i")` is "0.1+1i", `COMPLEX(1/3,1-2^-53)` "0.333333333333333+1i", `COMPLEX(2^31-1,1+2^-52)` "2147483647+1i", `IMDIV("5","-1.5+0.5i")` "-3-1i" (4 cells) and `IMPOWER("1000+1i",1)` "1000+1i"; ours writes "i" because `impl fmt::Display for Complex` (complex.rs) compares the 15-digit-rounded part with ±1. Excel compares the unrounded part. IMDIV's and IMPOWER's cells also need our imaginary part to be inexact there as Excel's is (Smith's division, IMPOWER's polar form): check both after the change; if ours is exactly -1 or 1, say which kernel differs in the Outcome and file it. Exactly 1 stays "i" (`IMSUB("9E307+1i","-9E307")` is "i", engineering.md T-2003).

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 10 known-failure lines in eval/baseline/gen-m9.txt, 7 formulas, 10 visibly off.
Examples:
- gen-m9/09e6a8028f5acfae Calc!A258 `=IMDIV("5","-1.5+0.5i")`: Excel "-3-1i", ours "-3-i"
- gen-m9/493bb194c8e0b577 Calc!A331 `=COMPLEX(1/3,1-2^-53)`: Excel "0.333333333333333+1i", ours "0.333333333333333+i"
- gen-m9/62c886e21a1042af Calc!A243 `=COMPLEX(0.1,1+2^-52,"i")`: Excel "0.1+1i", ours "0.1+i"
- gen-m9/65ff9a692665eba9 Calc!A291 `=COMPLEX(2^31-1,1+2^-52)`: Excel "2147483647+1i", ours "2147483647+i"
- gen-m9/74779cff2810ccf4 Calc!A379 `=IMPOWER("1000+1i",1)`: Excel "1000+1i", ours "1000+i"
- gen-m9/7dc52cfcbc8b4448 Calc!A247 `=COMPLEX(1+2^-52,1-2^-53,"j")`: Excel "1+1j", ours "1+j"
Cells:
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A258
- gen-m9/493bb194c8e0b577.case.json (1): Calc!A331
- gen-m9/62c886e21a1042af.case.json (1): Calc!A243
- gen-m9/65ff9a692665eba9.case.json (1): Calc!A291
- gen-m9/74779cff2810ccf4.case.json (2): Calc!A70, A379
- gen-m9/7dc52cfcbc8b4448.case.json (1): Calc!A247
- gen-m9/895a33aa8352864a.case.json (1): Calc!A8
- gen-m9/a1b4181c04804efc.case.json (1): Calc!A221
- gen-m9/ee70a5515d004a66.case.json (1): Calc!A256

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 10 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests for 1+2^-52, 1-2^-53, -1-2^-52 and exactly ±1.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.
