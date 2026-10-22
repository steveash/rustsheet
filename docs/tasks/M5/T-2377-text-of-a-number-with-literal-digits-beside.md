---
id: T-2377
title: "TEXT of a number with bare digits beside a date token: TEXT(3.14159,'42abc') is #VALUE!"
milestone: M5
lane: functions-text
status: done
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
- [x] A probes case with the cells above, oracle-filled (excel-graph), passes in full; number-formats.md states the rule.
- [x] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- The B2 Hijri calendar (T-1613).
- General among tokens (T-1614).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Outcome
Probe probes/text-format-digits-date-token (excel-graph, 132 cells: 42 codes on 3.14159, 100 and -3, plus text and
boolean values) shows that a bare digit 1-9 beside any date or time token in a section rejects the whole code. AM/PM alone
does not count, and quoted or escaped digits print. The rejection covers an unused number section too (`"0;4b"`) and text and
boolean values. Recorded in number-formats.md "Bare digits beside a date token". Implemented in `Parser::parse_part`
(`bare_digit`, `is_calendar_or_clock`); the section error carries `ErrorPart::whole_code`, which `format_number` checks
across the number sections. Other section errors keep their per-section behavior: making every bad section reject the code
regressed 56 gen-m10 cells whose `[Cyan]` section we do not parse (T-1737).
Eval delta: gen-m5 24 -> 23 (2b01cadfd03df4b5 A211 fixed). The new probe passes 131/132; C25 (`TEXT(-3,"b1")` is "-", ours
`#VALUE!`) is a new-coverage known failure, filed as T-2387. No regressions.
