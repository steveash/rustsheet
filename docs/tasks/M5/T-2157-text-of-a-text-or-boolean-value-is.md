---
id: T-2157
title: "TEXT of a text or boolean value is #VALUE! when the format code is invalid"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
campaign: visible
depends_on: [T-2202]
---

## Goal
TEXT is `#VALUE!` when its format code is invalid, whatever the value: a text or boolean value no longer bypasses the code check.

## Context
- Evidence (T-2202): `eval/suites/probes/cases/t2202-text-code-check-on-text` probes which codes are rejected and that the check covers text, empty text and booleans; its cells must pass here (see number-formats.md).
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2202 (research), which fills `probes/t2202-text-code-check-on-text`. Use that case; do not probe again. Its failing cells are this task's cells too.
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). `fn_text` (crates/rustsheet-engine/src/functions/text/common.rs) sends text and booleans to `format_text_value`
(formatter/format.rs), which never validates the number sections, so codes like "TRUE", "FALSE", "#N/A" and "HELLO " (a bare
`E`, an `N`, a date token beside a digit placeholder) return the value. Rule: docs/excel-semantics/number-formats.md "TEXT checks
the code before it looks at a text or boolean value" (added by this triage). `TEXT(2.5,"a*?b1")` (d1b251fd A120) is a number: probe
which token Excel rejects (`excel-probe`: `"a*?"`, `"*?b1"`, `"ab1"`, `"b1"` on 2.5 and on text) before changing the lexer.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 11 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/25768de48ae7f2f9 Calc!A395 | `=MIDB(TEXT(Data!B7,"TRUE"),2,3)` | "#VALUE!" | "x " |
| gen-m5/5704666097cbc5b2 Calc!A133 | `=TEXT((TEXT("",Data!A8)&Data!A4),IF(IF(TRUE,Data!B12),("#N/A">Data!$E7),(Data!$E8*Data!C$1)))` | "#VALUE!" | "TRUE" |
| gen-m5/651e713f5ae232ed Calc!A3 | `=TEXT("a",Data!E6)` | "#VALUE!" | "a" |
| gen-m5/6b831f5e2a20bc34 Calc!A320 | `=TEXT((ISBLANK(#NAME?)=Data!E4),Data!D12)` | "#VALUE!" | "FALSE" |
| gen-m5/8a387c26d182e73c Calc!A281 | `=TEXT(ISERR((Data!$B4-Data!B7)),CONCAT(TEXT(Data!B4,";;;"),IF(Data!E2,Data!C4)))` | "#VALUE!" | "FALSE" |
| gen-m5/8df4830f888f16ae Calc!A149 | `=REPLACEB(((Data!E$4&"?b")&TEXT(Data!C6,"#N/A")),6,6,Data!B1)` | "#VALUE!" | "123452.523?bTRUE" |
| gen-m5/a6fec430ac350c1a Calc!A331 | `=TEXT((""=-Data!B9),Data!$A1)` | "#VALUE!" | "FALSE" |
| gen-m5/c688ca172fe533dc Calc!A139 | `=TEXT((Data!B2>ISO.CEILING(Data!E7)),(TEXT("x~*",Data!D1)&(Data!D4&Data!C2)))` | "#VALUE!" | "FALSE" |
| gen-m5/d1b251fd6e9c4b4b Calc!A120 | `=TEXT(Data!B2,("a*"&(Data!E8&"1")))` | "#VALUE!" | "a " |
| gen-m5/d9fb1e918971111b Calc!A243 | `=LEFTB(((Data!D1&"x~*")&TEXT(Data!B3,Data!D7)))` | "#VALUE!" | "F" |
| gen-m5/f0f205865a3a4328 Calc!A262 | `=TEXT(Data!E11,LEFT(UPPER("Hello World"),6))` | "#VALUE!" | "TRUE" |

## Acceptance criteria
- [ ] These 11 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] number-formats.md entry marked implemented, with the `a*?b1` probe result.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
- Text values under valid 1-3 section codes (T-1130).
