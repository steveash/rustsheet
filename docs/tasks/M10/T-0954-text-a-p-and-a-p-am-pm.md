---
id: T-0954
title: "TEXT: a/p and A/P AM/PM markers in time formats (4 poi known failures)"
milestone: M10
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
campaign: semantics
depends_on: []
---

## Goal
Time formats with the short AM/PM marker `a/p` or `A/P` format like Excel: 12-hour clock, with a
single "a"/"p" (lowercase marker) or "A"/"P" (uppercase marker). Today they return `#VALUE!`.

## Context
Rule and evidence: docs/excel-semantics/number-formats.md, "`a/p` and `A/P` AM/PM markers".
Workbook `poi/DateFormatTests.xlsx` (desktop Mac Excel 16.0 cached values), `Tests!An` =
`=TEXT(Cn, Bn)`. All four are TypeMismatch with actual `#VALUE!`:

| cell | format | value | expected |
|---|---|---|---|
| A11 | `hh:mm:ss a/p` | 17816.607951388887 | "02:35:27 p" |
| A16 | `HH:MM:SS A/P` | 17816.607951388887 | "02:35:27 P" |
| A30 | `hh:mm:ss a/p` | 36191.1702084375 | "04:05:06 a" |
| A35 | `HH:MM:SS A/P` | 36191.1702084375 | "04:05:06 A" |

The workbook uses the 1904 date system, which the engine does not support yet. That does not matter
for these four cells, because they show only the time of day.

Cause: the formatter lexer (`crates/rustsheet-engine/src/formatter/lexer.rs`, the `'a' | 'A'` arm
that produces `Token::AMPM`) accepts only `AM/PM`, and the token does not record the case of the
short form. `AM/PM` must keep working (`TEXT(0.6079,"hh:mm:ss am/pm")` = "02:35:23 PM" today).
Desktop Excel prints "PM" for a lowercase `am/pm` too (DateFormatTests A13, A18 expected "...
PM"; those cells also need fractional seconds and stay out of scope).

## Acceptance criteria
- [ ] The 4 lines `F	poi/DateFormatTests.xlsx	Tests!A11`, A16, A30 and A35 disappear from
      eval/baseline/poi.txt, with no new F lines in any suite.
- [ ] Formatter unit tests for `a/p` and `A/P` on a morning and an afternoon time (the cells
      above). Noon and midnight have no Excel cell, so they are hand expectations that follow the
      engine's `AM/PM` handling ("12:00:00 p", "12:00:00 a").
- [ ] A unit test shows `hh:mm:ss am/pm` prints "PM" (rule from DateFormatTests A13 and A18; A13
      itself also needs fractional seconds, out of scope here), and the short marker keeps its case.
- [ ] The number-formats.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fractional seconds in clock formats (`h:m:s.00`, `hh:mm:ss.000 am/pm`; DateFormatTests A12,
  A13, A17, A18, A31, A32, A36, A37, A40, which return `#VALUE!` today). Same lexer area as T-0934
  root cause 1, so leave them to that work or a follow-up.
- The 1904 date system (every date-part cell in DateFormatTests and DateFormatNumberTests).

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format, so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Outcome
The `a/p` / `A/P` marker was already implemented by earlier formatter work (the four cells were no
longer in eval/baseline/poi.txt). This task added the missing unit tests (morning, afternoon, noon,
midnight, `am/pm` uppercase) and the `Implemented in:` line. Eval delta: none.
