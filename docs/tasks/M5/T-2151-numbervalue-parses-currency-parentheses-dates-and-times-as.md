---
id: T-2151
title: "NUMBERVALUE parses currency, parentheses, dates and times as VALUE does"
milestone: M5
lane: functions-text
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2152]
---

## Goal
NUMBERVALUE parses currency, parentheses, dates and times (also past 24 hours) as VALUE does, after applying its own separators.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_numbervalue` (crates/rustsheet-engine/src/functions/text/string_format.rs) strips separators and calls `parse_excel_number`,
so `"$1,000.50"`, `"(100)"`, `"2024-02-29"`, `"1900-02-29"`, `"25:00"` and `"12:30"` are `#VALUE!`. VALUE's parser
(`parse_formatted_number`, formatter/parse_formatted.rs) already reads these; reuse it after the separator step instead of adding a parser.
The separator step itself is T-2152 (first character only; group characters removed everywhere, even digits, e.g. group "5"
turns "25:00" into "2:00" = 0.0833, 34f69d85 A227); the two tasks touch the same function, so land T-2152 first or together.
Rule: docs/excel-semantics/text.md "NUMBERVALUE reads text as VALUE does, with its own separators" (added by this triage).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 24 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/399071ff6782ab16 Calc!A322 | `=DECIMAL(REPLACE(LEFT(Data!E12),6,ROUNDDOWN(42,5),(Data!D7&Data!A6)),NUMBERVALUE("25:00"))` | "#NUM!" | "#VALUE!" |
| gen-m5/901e91c09b347914 Calc!A368 | `=NUMBERVALUE("1900-02-29")` | 60.0 | "#VALUE!" |
| gen-m5/033e65b92653f425 Calc!A161 | `=NUMBERVALUE("$1,000.50")` | 1000.5 | "#VALUE!" |
| gen-m5/109fc90dceb50ba9 Calc!A144 | `=NUMBERVALUE("2024-02-29")` | 45351.0 | "#VALUE!" |
| gen-m5/1315170eef6c413e Calc!A93 | `=NUMBERVALUE("25:00",Data!D1,"abc")` | 1.0416666666666667 | "#VALUE!" |
| gen-m5/1962de140dae0111 Calc!A80 | `=NUMBERVALUE("1900-02-29")` | 60.0 | "#VALUE!" |
| gen-m5/34f69d858a39f497 Calc!A227 | `=NUMBERVALUE("25:00",PROPER(UNICHAR(12288)),RIGHT(IFS(Data!E9,Data!D$10,Data!E11,Data!$B6)))` | 0.08333333333333333 | "#VALUE!" |
| gen-m5/38b1b8e51dbad326 Calc!A340 | `=NUMBERVALUE("1900-02-29",ROMAN(1608,4))` | 60.0 | "#VALUE!" |
| gen-m5/4d099bea7898ebf1 Calc!A130 | `=NUMBERVALUE("25:00",RIGHT(Data!$A12,7),CLEAN("İ"))` | 1.0416666666666667 | "#VALUE!" |
| gen-m5/4d099bea7898ebf1 Calc!A261 | `=NUMBERVALUE("12:30")` | 0.5208333333333334 | "#VALUE!" |
| gen-m5/4febabc66cfdf9a1 Calc!A115 | `=NUMBERVALUE("12:30",SUBSTITUTE(UNICHAR(128512),(Data!A2&Data!B$11),Data!$A4),Data!D6)` | 0.5208333333333334 | "#VALUE!" |
| gen-m5/6a511beadb56b932 Calc!A282 | `=NUMBERVALUE("$1,000.50")` | 1000.5 | "#VALUE!" |
| gen-m5/6b831f5e2a20bc34 Calc!A227 | `=NUMBERVALUE("25:00")` | 1.0416666666666667 | "#VALUE!" |
| gen-m5/6ba5e4aceeef8209 Calc!A177 | `=ISEVEN(NUMBERVALUE("25:00"))` | false | "#VALUE!" |
| gen-m5/7d1a5d223624fc70 Calc!A295 | `=SUMX2PY2(Data!D10:E11,(ROUNDDOWN(0.1,4)/NUMBERVALUE("$1,000.50")))` | "#N/A" | "#VALUE!" |
| gen-m5/7d1a5d223624fc70 Calc!A377 | `=NUMBERVALUE("25:00")` | 1.0416666666666667 | "#VALUE!" |
| gen-m5/aac6230c2fb40e1e Calc!A82 | `=NUMBERVALUE("$1,000.50",Data!D3)` | 1000.5 | "#VALUE!" |
| gen-m5/ae9ed30812c4d62c Calc!A285 | `=NUMBERVALUE("(100)",ROMAN(1658))` | -100.0 | "#VALUE!" |
| gen-m5/b0ecb7ca78460b77 Calc!A145 | `=ACOTH(-NUMBERVALUE("$1,000.50"))` | -0.000999500582709095 | "#VALUE!" |
| gen-m5/b237245054b4fedb Calc!A128 | `=NUMBERVALUE("25:00",VALUETOTEXT(Data!D$9%,1),Data!E$9)` | 0.20833333333333334 | "#VALUE!" |
| gen-m5/c0651f0e0fb544d0 Calc!A340 | `=NUMBERVALUE("12:30")` | 0.5208333333333334 | "#VALUE!" |
| gen-m5/c688ca172fe533dc Calc!A170 | `=NUMBERVALUE("2024-02-29","a")` | 45351.0 | "#VALUE!" |
| gen-m5/eb942f4a59b1c004 Calc!A115 | `=NUMBERVALUE("25:00",IF(FALSE(),((-0.0001)-Data!C7)),Data!E1)` | 1.0416666666666667 | "#VALUE!" |
| gen-m5/ee1667411bbe2ccc Calc!A88 | `=NUMBERVALUE("25:00",IFNA((3.14159<>Data!E12),((-0.0001)+Data!A$9)),Data!$B3)` | 1.0416666666666667 | "#VALUE!" |

## Acceptance criteria
- [ ] These 24 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] text.md entry marked implemented.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
- Array lifting (T-2160).
