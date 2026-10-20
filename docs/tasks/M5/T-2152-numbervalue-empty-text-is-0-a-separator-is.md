---
id: T-2152
title: "NUMBERVALUE: empty text is 0, a separator is its first character, an empty separator is #VALUE!"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
NUMBERVALUE reads empty text (or text emptied by the group separator) as 0, uses only the first character of each separator, removes every occurrence of the group character, and is `#VALUE!` for an empty separator.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_numbervalue` (string_format.rs) returns `#VALUE!` for empty text and replaces whole separator strings (`"0.2"`, `"abc"`), and
accepts an empty group separator. Rule: docs/excel-semantics/text.md "NUMBERVALUE reads text as VALUE does, with its own
separators" (added by this triage). 1b30e4d0 A392 (`NUMBERVALUE(100,"abc",100)` inside `YEAR`) passes with either reading of
"100" once the separators are first characters.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 12 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/1b30e4d075b252ea Calc!A392 | `=SQRTPI(YEAR(NUMBERVALUE(Data!A7,Data!B10,Data!A7)))` | 77.25947218186653 | "#VALUE!" |
| gen-m5/5704666097cbc5b2 Calc!A261 | `=NUMBERVALUE(Data!D2,RIGHT(Data!A2,5))` | "#VALUE!" | 0.3 |
| gen-m5/5704666097cbc5b2 Calc!A392 | `=NUMBERVALUE(Data!E8)` | 0.0 | "#VALUE!" |
| gen-m5/6b831f5e2a20bc34 Calc!A358 | `=NUMBERVALUE(IFERROR(MID("?b",4,5),MMULT(Data!B2:D4,0)))` | 0.0 | "#VALUE!" |
| gen-m5/8df4830f888f16ae Calc!A356 | `=NUMBERVALUE(RIGHT((Data!E7&""),1))` | 0.0 | "#VALUE!" |
| gen-m5/a6fec430ac350c1a Calc!A32 | `=NUMBERVALUE(" 42 ","#N/A",Data!D9)` | "#VALUE!" | 42.0 |
| gen-m5/aac6230c2fb40e1e Calc!A344 | `=NUMBERVALUE(T((Data!D2-Data!$A$8)),VALUETOTEXT((Data!$D1/Data!A7),0))` | 0.0 | "#VALUE!" |
| gen-m5/b237245054b4fedb Calc!A390 | `=NUMBERVALUE(LEFTB(PROPER(Data!B5),1.005),LEFT((Data!A10&Data!B11)),CONCATENATE("ABC","?b",TRIM("#...` | 0.0 | "#VALUE!" |
| gen-m5/bbbd316c346ce645 Calc!A76 | `=NUMBERVALUE(IFNA(Data!E7,RIGHT("#N/A")),(Data!D9&Data!D5),("ABC"&LEFT(Data!$B2)))` | "#VALUE!" | 0.1 |
| gen-m5/c688ca172fe533dc Calc!A301 | `=NUMBERVALUE(REPT(("abc"&"abc"),(Data!C$11*Data!$D9)),Data!E1,((Data!A9&Data!E5)&MID(Data!B6,5,6)))` | 0.0 | "#VALUE!" |
| gen-m5/ecd1b828aebad1e8 Calc!A60 | `=NUMBERVALUE(RIGHT("1.5",(Data!A$3/Data!B1)))` | 0.0 | "#VALUE!" |
| gen-m5/ee1667411bbe2ccc Calc!A170 | `=DEGREES(SUMX2PY2(NUMBERVALUE(Data!$C1),Data!B4:B4))` | 8.73278137242291e+17 | "#VALUE!" |

## Acceptance criteria
- [ ] These 12 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Currency, parentheses, dates and times (T-2151); array lifting (T-2160).
