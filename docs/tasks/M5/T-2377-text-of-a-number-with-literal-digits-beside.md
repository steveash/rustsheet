---
id: T-2377
title: "TEXT of a number with bare digits beside a date token: TEXT(3.14159,'42abc') is #VALUE!"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
TEXT of a number with a code that mixes bare digits and a date token matches Excel (`TEXT(3.14159,"42abc")` is `#VALUE!`).

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). `TEXT(3.14159,"42abc")` is `#VALUE!` in Excel; ours prints "42a43c" (`b` is the Buddhist-era year, "43"). `"ABC"` alone is
"A43C" in Excel and a bare digit alone is a literal (`TEXT(5,"2")` = "2"), so the rejection comes from the mix. Observation:
docs/excel-semantics/number-formats.md, paragraph after "Buddhist-era year, era and calendar tokens" (added by this triage).
Needs a probe first (`excel-probe` skill, excel-graph): `TEXT(3.14159,c)` for c in "42abc", "4b", "b4", "4 b", "1b", "9b",
"0b", "#b", "4d", "d4", "4yy", "4mmm", "4h", "4:mm", "x4b", "\4b", "\"4\"b", "42", "4a" and "4c"; then a text and a boolean
value on "42abc" (T-2202 found the check all-or-nothing). Record the rule, then implement it in the format lexer/parser
(formatter/lexer.rs, formatter/parser.rs).

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/2b01cadfd03df4b5 Calc!A211 | `=LENB(TEXT((Data!A$2+Data!$B9),(Data!C10&"abc")))` | #VALUE! | 6 |

## Acceptance criteria
- [ ] A probes case with the cells above, oracle-filled (excel-graph), passes in full; number-formats.md states the rule.
- [ ] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The B2 Hijri calendar (T-1613).
- General among tokens (T-1614).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.
