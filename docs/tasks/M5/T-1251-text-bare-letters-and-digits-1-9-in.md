---
id: T-1251
title: "TEXT: bare letters and digits 1-9 in a number format are literals, and an empty format is empty text (3 gen-m6 cells)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
TEXT on a number treats unquoted letters (such as `ü`, `x`, `a`) and the digits 1-9 in a format as literal characters, and an empty format gives empty text, as Excel does, instead of `#VALUE!`.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09):
- gen-m6/2c1a6e93a1974f48 Calc!A15 `=(TEXT(PI(),(Data!A5&Data!$A7))&"x~*")` (A5 = "ü", A7 blank, so the format is `ü`):
  Excel "üx~*", engine `#VALUE!`. So `TEXT(PI(),"ü")` = "ü".
- gen-m6/63012866a9d8b7db Calc!A3 `=TEXT(Data!C11,IF(NOT(TRUE),..,TEXT(Data!C12,Data!D1)))` with C11 = 1.005, C12 = -1,
  D1 = 2.5: Excel "-211.5", engine `#VALUE!`. The inner `TEXT(-1,"2.5")` must give "-21.5" and the outer
  `TEXT(1.005,"-21.5")` "-211.5": digits 1-9 print as literals and, with a decimal point but no digit placeholder,
  the integer part goes right before the point (inferred from this one cell; probe it).
- gen-m6/17223361bef5b5cb Calc!A12 `=(INDIRECT(TEXT((-0.0001),Data!$B8),Data!A3)&ASC(..))` with B8 blank: Excel `#REF!`,
  engine `#VALUE!`. `TEXT(-0.0001,"")` is not an error in Excel (INDIRECT then fails with `#REF!`); number-formats.md
  "An empty format string returns empty text" has only probed a blank value.
The engine's format lexer rejects these formats (`ParsePart::Error` in crates/rustsheet-engine/src/formatter/format.rs,
lexer in formatter/lexer.rs) and `fn_text` (functions/text/common.rs) turns that into `#VALUE!`.
Semantics: docs/excel-semantics/number-formats.md "Bare letters and digits in a number format" (added by T-1239's triage).
Probe first (excel-probe skill): `TEXT(PI(),"ü")`, `TEXT(PI(),"x")`, `TEXT(-PI(),"x")`, `TEXT(5,"2")`, `TEXT(-1,"2.5")`,
`TEXT(1.005,"-21.5")`, `TEXT(12.3,".")`, `TEXT(12.3,"a.b")`, `TEXT(0,"")`, `TEXT(-0.0001,"")`, `TEXT(5,"")`, and which
letters are not literals (date/time tokens such as `d`, `m`, `y`, `h`, `s`, `e`, `g` (General), `b`).

## Acceptance criteria
- [ ] An excel-graph probe case covers the formats above and passes; number-formats.md records the rule (VERIFIED).
- [ ] gen-m6/2c1a6e93a1974f48 A15, gen-m6/63012866a9d8b7db A3 and gen-m6/17223361bef5b5cb A12 pass in
      `./target/release/rustsheet-eval show <case>`, with no new failing gen-m6 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other TEXT format families already tasked (T-0953 £/¥/|, T-0982 fractions, T-1129 sign of values rounding to zero).
