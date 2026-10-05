---
id: T-0953
title: "TEXT: £, ¥ and | are literal characters in number formats (13 poi known failures)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT` accepts `£`, `¥` and `|` unquoted in a number format and prints them as written, like
Excel, instead of returning `#VALUE!`.

## Context
Rule and evidence: docs/excel-semantics/number-formats.md, "`£`, `¥` and `|` print as literals
without quotes". Workbook `poi/NumberFormatTests.xlsx` (desktop Mac Excel 16.0 cached values),
`Tests!An` = `=TEXT(Cn, Bn)`. All failing cells are TypeMismatch with actual `#VALUE!`:

| cell | format | value | expected |
|---|---|---|---|
| A5 | `£000.00` | 12.3 | "£012.30" |
| A18 | `¥#.00" after"` | 12.3 | "¥12.30 after" |
| A19 | `"before "¥#.00` | 12.3 | "before ¥12.30" |
| A21 | `¥#.00" after"` | -12.3 | "-¥12.30 after" |
| A22 | `"before "¥#.00` | -12.3 | "-before ¥12.30" |
| A107 | `\|?.?\|` | 1.1 | "\|1.1\|" |
| A108 | `\|??.?\|` | 1.1 | "\| 1.1\|" |
| A109 | `\|?.??\|` | 1.1 | "\|1.1 \|" |
| A110 | `\|??.??\|` | 1.1 | "\| 1.1 \|" |
| A113 | `\|?.?\|` | -1.1 | "-\|1.1\|" |
| A114 | `\|??.?\|` | -1.1 | "-\| 1.1\|" |
| A115 | `\|?.??\|` | -1.1 | "-\|1.1 \|" |
| A116 | `\|??.??\|` | -1.1 | "-\| 1.1 \|" |

(The `\|` in the table is markdown escaping: the format is a plain `|`.)

Cause: the formatter lexer (`crates/rustsheet-engine/src/formatter/lexer.rs`, `next_token`) has a
fixed list of characters that are literals without quotes (`$`, `€`, `(`, `)`, ...). Any other
character is `Token::ILLEGAL`, so `TEXT` returns `#VALUE!`. Add `£`, `¥` and `|` to that list. Do
not add characters the evidence does not cover. A spec case confirms `€` already works
(`TEXT(12.3,"€0.00")` → "€12.30").

Expect side effects: `|` also appears in other POI formats that fail today for other reasons
(fractions in NumberFormatTests, `|#.#|E+|#|` in NumberFormatApproxTests, T-0937). Those cells may
start to render, or they may change their wrong value. Either way, no new F line may appear. If a
case's checked-cell count or a failure's verdict changes, say so in the PR.

## Acceptance criteria
- [ ] The 13 lines `F	poi/NumberFormatTests.xlsx	Tests!A5`, A18, A19, A21, A22, A107, A108,
      A109, A110, A113, A114, A115 and A116 disappear from eval/baseline/poi.txt, with no new F
      lines in any suite.
- [ ] Formatter unit tests for one `£`, one `¥` with a quoted literal and a negative value, and one
      `|` case from the table.
- [ ] The number-formats.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Grouping commas (T-0952), scientific formats with literals after `E` (T-0937), fraction
  formats.
- Any other currency symbol or punctuation without Excel evidence. Use the `excel-probe` skill and
  file a follow-up if needed.
