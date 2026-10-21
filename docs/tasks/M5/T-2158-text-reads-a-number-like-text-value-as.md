---
id: T-2158
title: "TEXT reads a number-like text value as its number and prints control characters in the code"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
TEXT formats a number-like text value as its number (`TEXT("1","0,000.0")` = "0,001.0", `TEXT("$3.14","h:mm:ss.00")` = "3:21:36.00") and prints control characters in the code as literals.

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_text` (functions/text/common.rs) keeps text values as text, and the format lexer (formatter/lexer.rs) rejects CHAR(1) and
CHAR(10). Rule: docs/excel-semantics/number-formats.md "TEXT reads a number-like text value as its number; control characters
are literals" (added by this triage). Coerce with VALUE's parser (`parse_formatted_number`, formatter/parse_formatted.rs), not a new one.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 4 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/901e91c09b347914 Calc!A210 | `=TEXT(IF(IF(Data!A$10,1,Data!$B5),"1"),"0,000.0")` | "0,001.0" | "1" |
| gen-m5/eb942f4a59b1c004 Calc!A54 | `=TEXT(DOLLAR(3.14159),"h:mm:ss.00")` | "3:21:36.00" | "$3.14" |
| gen-m5/c0651f0e0fb544d0 Calc!A336 | `=TEXT(Data!D4,UNICHAR(N(Data!E12)))` | "-\u0001" | "#VALUE!" |
| gen-m5/d586a1e0c9896942 Calc!A38 | `=TEXT(100,LOWER(CHAR(10)))` | "\n" | "#VALUE!" |

## Acceptance criteria
- [ ] These 4 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
- Date-like and percent text values (unprobed; note them in number-formats.md if the coercion covers them).

## Outcome
The format lexer prints control characters as literals and `fn_text` reads number-like text with VALUE's parser. gen-m5 known failures 16 -> 12 (the 4 owned cells fixed), no regressions elsewhere. Date and percent text now also coerce (VALUE's parser); still unprobed.
