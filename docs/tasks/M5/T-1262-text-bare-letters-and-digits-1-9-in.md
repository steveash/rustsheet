---
id: T-1262
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

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/6db767d720c5b55a Calc!A30 `=TEXT(Data!A3,"ABC")` with A3 = 100: Excel "A43C", engine `#VALUE!`. "A" and "C" are literals but "B" is not:
  it prints "43", which fits a two-digit Buddhist-era year (100 is 1900-04-09, year 1900 + 543 = 2443). Include `B`, `BB`, `BBBB`,
  `b` and `B1`/`B2` in the probe (number-formats.md "Bare letters and digits in a number format").

More evidence (T-1450, probes/text-format-unquoted-letters, excel-graph 2026-10-14): which letters are literals is now probed for every ASCII letter after `0` and after `d`; the table is in number-formats.md "Unquoted ASCII letters in a number format". Of its 111 cells 75 fail today: 68 literal letters, plus the unimplemented `b`/`e`/`g` date tokens (`db`, `de`, `dg`, `dG`), `TEXT(1,"0\:\:")` (we collapse the escaped colons, Excel prints "1::") and `0:a:` / `0::a`. Make them pass too, or split what does not fit into a new task. Check with `./target/release/rustsheet-eval show probes/text-format-unquoted-letters.case.json`.

## Acceptance criteria
- [ ] An excel-graph probe case covers the formats above and passes; number-formats.md records the rule (VERIFIED).
- [ ] gen-m6/2c1a6e93a1974f48 A15, gen-m6/63012866a9d8b7db A3 and gen-m6/17223361bef5b5cb A12 pass in
      `./target/release/rustsheet-eval show <case>`, with no new failing gen-m6 cell and no new known failures in eval/baseline/.
- [ ] Every cell of probes/text-format-unquoted-letters passes.
- [ ] gen-m7/6db767d720c5b55a A30 passes. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other TEXT format families already tasked (T-0953 £/¥/|, T-0982 fractions, T-1129 sign of values rounding to zero).
