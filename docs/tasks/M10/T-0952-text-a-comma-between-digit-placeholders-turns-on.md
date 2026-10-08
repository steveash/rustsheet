---
id: T-0952
title: "TEXT: a comma between digit placeholders turns on thousands grouping; #,, keeps the sign (13 poi known failures)"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Number formats with a `,` between digit placeholders group the integer part in threes, as Excel
does. This must work wherever the comma sits (`0,0000000`, `?,?????????`), not only in the
`#,##0` shape. A trailing scaling `,` keeps the number's sign.

## Context
Rule and evidence: docs/excel-semantics/number-formats.md, "A comma between digit placeholders
turns on thousands grouping". Workbook `poi/NumberFormatTests.xlsx` (desktop Mac Excel 16.0 cached
values). `Tests!An` is `=TEXT(Cn, Bn)`. Failing cells, all TextMismatch:

| cell | format | value | expected | actual |
|---|---|---|---|---|
| A44 | `?,???????` | 1234567 | " 1,234,567" | " ,123,4567" |
| A45 | `?,????????` | 1234567 | "  1,234,567" | " , 12,34567" |
| A46, A112 | `?,?????????` | 1234567 | "    1,234,567" | " ,  1,234567" |
| A67 | `0,0000000` | 1234567 | "01,234,567" | "0,123,4567" |
| A68 | `0,00000000` | 1234567 | "001,234,567" | "0,012,34567" |
| A69 | `0,000000000` | 1234567 | "0,001,234,567" | "0,001,234567" |
| A80 | `?,???????` | -1234567 | "- 1,234,567" | "- ,123,4567" |
| A81 | `?,????????` | -1234567 | "-  1,234,567" | "- , 12,34567" |
| A82, A118 | `?,?????????` | -1234567 | "-    1,234,567" | "- ,  1,234567" |
| A95 | `#,,` | -1234567 | "-1" | "1" |
| A335 | `$000,000,000` | 314159 | "$000,314,159" | "$000,314159" |

The formatter puts the comma where it appears in the format instead of treating it as a grouping
flag. It also drops the sign when `#,,` scales the value. Code: `crates/rustsheet-engine/src/formatter/`
(`Token::Comma` in `lexer.rs`; grouping and scaling in `parser.rs`/`format.rs`). `#,##0`
(1234567 → "1,234,567") already passes and must keep passing. Note the `?` case: a separator inside
the padded part becomes a space (A46).

## Acceptance criteria
- [ ] The 13 lines `F	poi/NumberFormatTests.xlsx	Tests!A44`, A45, A46, A67, A68, A69, A80, A81,
      A82, A95, A112, A118 and A335 disappear from eval/baseline/poi.txt, with no new F lines in
      any suite.
- [ ] Formatter unit tests cover a `0`-padded, a `?`-padded and a negative case from the table,
      and `#,,` on a negative number.
- [ ] The number-formats.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fraction formats (`#/#`, `?/?`) in the same workbook: most of its other known failures.
- Literal characters such as `£`, `¥`, `|` (T-0953).
