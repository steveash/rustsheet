---
id: T-1361
title: "TEXTSPLIT, TEXTBEFORE and TEXTAFTER read a multi-cell delimiter range as a delimiter list (blank cell, number and boolean delimiters)"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A multi-cell range (any shape) passed as the delimiter of TEXTSPLIT (col and row delimiters), TEXTBEFORE or TEXTAFTER is read as a list of delimiters as Excel does; how a blank cell and a number or boolean element match is probed first (UNVERIFIED in text.md).

## Context
T-1275's gen-m7 suite (excel-graph sidecars, 2026-10-11); docs/excel-semantics/text.md "TEXTSPLIT, TEXTBEFORE and TEXTAFTER delimiter ranges":
- gen-m7/0e40208f51c2454c Calc!A17 `=IF(INDIRECT(TEXTAFTER(Data!$A5,Data!A5:D6)),..)`, A5 = "TRUE": Excel `#REF!` (TEXTAFTER gives text, inferred "";
  INDIRECT("") is `#REF!`), engine `#VALUE!`.
- gen-m7/8ed8a680e9794b77 Calc!A17 `=TEXTBEFORE("a*",Data!C1:D11)` (C1:D11 has blank cells): Excel "" (the "a" in C5 matches at the start, so this cell does not show the blank rule), engine `#VALUE!`.
- gen-m7/8ed8a680e9794b77 Calc!A26 `=(TEXTBEFORE("",Data!C3:D3)+LN(INDEX(Data!C4:E5,1)))`, C3:D3 = TRUE, 1E+15: Excel `#N/A`
  (no delimiter found), engine `#VALUE!` (`TEXTBEFORE("",TRUE)` alone is already `#N/A`).
- gen-m7/4629e931955aa4b8 Calc!A19 `=(MATCH(TEXTSPLIT("#N/A",Data!$B4,Data!B11:B12),Data!E8:E10)-LOG10(0))`, B11:B12 = 1E+15,
  TRUE: Excel `#NUM!`, engine `#VALUE!`.
Engine today: a one-row range of text delimiters works (`TEXTAFTER("TRUE",Data!A5:D5)` in 0e40208f51c2454c is ""), a
multi-row range or one holding a blank or boolean is `#VALUE!` (`TEXTBEFORE("a*",Data!C4:D4)` with C4 blank, D4 FALSE).
Code: `fn_textbefore`/`fn_textafter` in crates/rustsheet-engine/src/functions/text/common.rs, `fn_textsplit` in
functions/text/textsplit.rs; find the shared delimiter reader and fix it once. Array-constant delimiters (`#N/IMPL!` for
`TEXTAFTER("TRUE",{"TRUE","x"})`) and lifting the text over a range are T-1272; check which lands first and do not
duplicate the list reader.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/0c7fc447bc2eebfd Calc!A6 `=(((Data!E12*Data!E4)-TEXTBEFORE(Data!C6,Data!A9:D10,1))/Data!E1)` (text 2; delimiters 0, "?b", blank, 1.005, FALSE and three blanks): Excel 100000000002, so TEXTBEFORE is "2" (the whole text), engine `#VALUE!`.
- gen-m9/4df1b22d3ed63f58 Calc!A14 `=(Data!D5&(" x "&TEXTBEFORE(Data!C$8,Data!C11:D11,Data!A5)))` (text 42; delimiters 1E+15 and a blank; instance_num 1E+15): Excel " x ", so TEXTBEFORE is "", engine `#VALUE!`.
- gen-m9/4df1b22d3ed63f58 Calc!A30 `=TEXTBEFORE((""&("1"&Data!A11)),Data!A8:C11)` ("1TRUE"; a 4x3 range holding 1 and TRUE): Excel "", engine `#VALUE!`.
- gen-m9/3ec66461400faf2f Calc!A3 `=TEXTBEFORE(Data!C6,Data!D4:E5,-1)` (TRUE; 2.5, 1E-15, "x", "1"): Excel `#N/A`, engine `#VALUE!`.
- gen-m9/696af2c8cee2f57e Calc!A23 `=TEXTAFTER(Data!$E$3,Data!D4:E10)` (123456789.123; numbers, text and FALSE): Excel `#N/A`, engine `#VALUE!`.
- gen-m9/95efb37a9874f82e Calc!A25 `=TEXTBEFORE(" x ",Data!D10:E11)`: Excel `#N/A`, engine `#VALUE!`.
- The first two contradict "a blank cell is the empty delimiter" (that would give "" in A6) and the instance_num bound (A14): add `TEXTBEFORE("2",A1:B1)` with A1 blank and B1 "q", and `TEXTBEFORE("42",A1:B1,1E+15)`, to the probe (text.md entry updated).

- gen-m7/851eb2099ae3f805 Calc!A25 `=TEXTAFTER(" x ",Data!A6:A10,Data!A9)`: Excel `#VALUE!`. Ours `#VALUE!` (a TEXTBEFORE/TEXTAFTER delimiter range is read as one text, kept so by T-1272). An early list reading of the range, with its blanks dropped, gave `#N/A`; the rule for a range holding blanks, numbers or booleans must match this cell. gen-m9/3ec66461400faf2f A3 and 95efb37a9874f82e A25 pass once ranges are lists.

## Acceptance criteria
- [ ] gen-m7/0e40208f51c2454c A17, gen-m7/8ed8a680e9794b77 A17 and A26 and gen-m7/4629e931955aa4b8 A19 pass in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] An excel-probe case settles the UNVERIFIED parts first: `TEXTBEFORE("xyz",A1:B1)` with A1 blank and B1 "q", `TEXTBEFORE("a1b",A1:B1)` with A1 = 1, `TEXTAFTER("a TRUE b",A1:B1)` with A1 = TRUE; text.md updated.
- [ ] Unit tests: a 2x2 delimiter range, a range with a blank cell, a range with a number and a boolean, per the probe.
- [ ] gen-m9/0c7fc447bc2eebfd A6, 4df1b22d3ed63f58 A14 and A30, 3ec66461400faf2f A3, 696af2c8cee2f57e A23 and 95efb37a9874f82e A25 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Handoff
Escalated by a sonnet run without implementing (work-loop step 5b: unclear evidence). What I found, not tried in code:
- The rule for blank, number and boolean delimiter cells is genuinely unknown and the listed cells constrain it in conflicting ways
  (gen-m9/0c7fc447bc2eebfd A6 and 4df1b22d3ed63f58 A14 contradict "blank = empty delimiter"; gen-m7/851eb2099ae3f805 A25 shows a
  range holding blanks is `#VALUE!` while gen-m9 95efb37a9874f82e A25 gives `#N/A`). The first step is the excel-probe case in the
  acceptance criteria (needs the oracle), then infer one rule that fits every cell listed in Context.
- Next: run the probe, write the rule into docs/excel-semantics/text.md, fix the shared delimiter reader once
  (`fn_textbefore`/`fn_textafter` in functions/text/common.rs, `fn_textsplit` in functions/text/textsplit.rs), check T-1272 first.
