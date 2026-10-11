---
id: T-1346
title: "TEXTSPLIT, TEXTBEFORE and TEXTAFTER read a multi-cell delimiter range as a delimiter list (blank cell, number and boolean delimiters)"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
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

## Acceptance criteria
- [ ] gen-m7/0e40208f51c2454c A17, gen-m7/8ed8a680e9794b77 A17 and A26 and gen-m7/4629e931955aa4b8 A19 pass in `./target/release/rustsheet-eval show gen-m7/<case>.case.json`, with no new failing gen-m7 cell and no new known failures in eval/baseline/.
- [ ] An excel-probe case settles the UNVERIFIED parts first: `TEXTBEFORE("xyz",A1:B1)` with A1 blank and B1 "q", `TEXTBEFORE("a1b",A1:B1)` with A1 = 1, `TEXTAFTER("a TRUE b",A1:B1)` with A1 = TRUE; text.md updated.
- [ ] Unit tests: a 2x2 delimiter range, a range with a blank cell, a range with a number and a boolean, per the probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
