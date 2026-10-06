---
id: T-0981
title: "TEXT: fractional seconds (s.0, s.00, s.000) in clock-time formats (5 poi known failures)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Date/time formats with fractional seconds after a plain `s`/`ss` (`hh:mm:ss.000`, `h:m:s.00`) render
like Excel instead of returning `#VALUE!`.

## Context
Evidence: `poi/DateFormatTests.xlsx` (desktop Excel cached values, Microsoft Macintosh Excel 16.0),
`Tests!An` = `=TEXT(Cn, Bn)`. The workbook uses the 1904 date system, which the engine does not
support, but these cells show only the time of day, so it does not matter. All TypeMismatch with
actual `#VALUE!`:

| cell | format | value | expected |
|---|---|---|---|
| A13 | `hh:mm:ss.000 am/pm` | 17816.607951388887 | "02:35:27.000 PM" |
| A18 | `HH:MM:SS.000 AM/PM` | 17816.607951388887 | "02:35:27.000 PM" |
| A32 | `hh:mm:ss.000 am/pm` | 36191.1702084375 | "04:05:06.009 AM" |
| A37 | `HH:MM:SS.000 AM/PM` | 36191.1702084375 | "04:05:06.009 AM" |
| A40 | `h:m:s.00` | 36191.1702084375 | "4:5:6.01" |

The value's time of day is 04:05:06.009 (A32) and 14:35:27.000 (A13); `.00` rounds the fraction to
hundredths (A40 "6.01"). Lowercase `am/pm` prints "PM" (already works: A46 `d "days" h am/pm` gives
"... 2 PM" today; docs/excel-semantics/number-formats.md, "`a/p` and `A/P` AM/PM markers").

Four more cells combine fractional seconds with the short `a/p`/`A/P` marker (T-0954): A12
`h:m:s.000 A/P` → "2:35:27.000 P", A17 `H:M:S.00 a/p` → "2:35:27.00 p", A31 `h:m:s.00 A/P` → "4:5:6.01 A",
A36 `H:M:S.00 a/p` → "4:5:6.01 a".

Code: `crates/rustsheet-engine/src/formatter/` (`lexer.rs` date tokens, `format.rs` date rendering).
T-0934 adds fractional seconds after elapsed tokens (`[s].000`); if it has merged, reuse its
fraction rendering; if not, write the helper so T-0934 can use it (one routine for both).
Rule: docs/excel-semantics/number-formats.md, "Fractional seconds in clock formats".

## Acceptance criteria
- [ ] The 5 lines `F	poi/DateFormatTests.xlsx	Tests!A13`, A18, A32, A37 and A40 disappear from
      eval/baseline/poi.txt with no new F lines in any suite.
- [ ] If T-0954 has merged first, A12, A17, A31 and A36 disappear too; otherwise the Outcome says
      they wait for T-0954.
- [ ] Formatter unit tests for `hh:mm:ss.000` and `h:m:s.00` on the two values above.
- [ ] The number-formats.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The date part of these workbooks (1904 date system: DateFormatTests A2:A9, A14, A19:A28, ...,
  DateFormatNumberTests).
- Elapsed `[h]`/`[m]`/`[s]` formats (T-0934).
