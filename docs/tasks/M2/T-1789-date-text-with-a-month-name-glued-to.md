---
id: T-1789
title: "Date text with a month name glued to digits parses (July1, January0, 1JUN2021)"
milestone: M2
lane: functions-date
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1788]
---

## Goal
Date text with a month name glued to digits (`"July1"`, `"January0"`, `"1JUN2021"`) parses as Excel
reads it, so MONTH, DATEVALUE and the other date-text readers stop returning `#VALUE!` for it.

## Context
Largest cause in corpus-bench after the oracle artifact (T-0433): 623 failing cells in 3 cases, all
`#VALUE!` (or the IFERROR fallback `""`) where Excel gives a date (semantics and cells:
docs/excel-semantics/dates.md, "Date text with a month name glued to digits"):
- corpus-bench/55392 (526 cells): `Monthly Expenses Summary!E6:P30` `=IFERROR(SUMPRODUCT(...(MONTH('Journal Entries'!$B$6:$B$3000)=MONTH(E$4&0))...),"")`, E4 = `"January"`: expected numbers (G6 = 35000), engine `""`; the row/column totals (Q6, G35, H48, ...) follow.
- corpus-bench/CF_18814 (64 cells): `attendance-Dec!E1` `=DATEVALUE("1"&B1&B2)` with `"JUN"`, 2021: expected 44348, engine `#VALUE!`; C3:AG3 and the rest follow.
- corpus-bench/1726 (33 cells): `July-KPIs!J12` `=DATE(F2,MONTH(D2&1),E2)` with D2 = `"July"`: expected 45483, engine `#VALUE!`; A6:A36 follow.
Inspect with `./target/release/rustsheet-eval show corpus-bench/<id>/1_<id>_input.xlsx`.
Code: `formatter::date_text::parse_date` (shared by DATEVALUE, VALUE, number coercion and criteria).
The probe T-1788 settles which digits are day and year; implement its rule, not a guess.
T-1788's rule (dates.md, probes/date-text-glued-month): a glued month and one number read as with a space
(the year-less "Month d" rule, so `"July32"` is 1932-07-01 and `"Jul2026"` 2026-07-01), day-month(-year)
parses glued on either side, and a month glued to a day that a year follows (`"July1,2026"`, `"Jun1 2021"`)
stays `#VALUE!`.

## Acceptance criteria
- [x] The `F` lines of corpus-bench/55392, CF_18814 and 1726 leave eval/baseline/corpus-bench.txt (1726 keeps its line only for `July-KPIs!A1`, T-1089), with no new ones.
- [x] The probe case of T-1788 passes; unit tests in `date_text` cover each glued form.
- [x] dates.md entry updated with "Implemented in"; `make check` green; reviewers approve.

## Out of scope
- Other date-text shapes not evidenced by T-1788.

## Outcome
`parse_date` splits a whitespace word whose month name is glued to digits (`split_glued_month`: optional
digits, a month name, optional digits) into words and reads it with the existing spaced grammar; a word whose
month is glued to a following number must be the last word (`"Jun1 2021"`, `"Jul1 , 2026"` are `#VALUE!`).
Eval: probes/date-text-glued-month 30 -> 178/178 (148 fixed); corpus-bench 55392 16118 -> 16644/16644,
CF_18814 27 -> 91/91, 1726 66 -> 99/100 (only `July-KPIs!A1`, T-1089), so 623 corpus cells fixed and
corpus-bench known-failure cases 19 -> 17; 0 regressions.
