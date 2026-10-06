---
id: T-0256
title: "LEFT/RIGHT/MID count arguments: coerce numeric text and booleans (6 poi known failures)"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The number arguments of LEFT, RIGHT and MID (num_chars, start_num) accept numeric text and booleans
like Excel does: numeric text is read as a number, TRUE is 1 and FALSE is 0. Non-numeric text stays
`#VALUE!`.

## Context
Evidence: `poi/FormulaEvalTestData_Copy.xlsx` (desktop Excel cached values), sheet EverythingTests. Six
known failures, all TypeMismatch with actual `#VALUE!`:

| cell | formula | inputs | expected |
|---|---|---|---|
| E824 | `LEFT("a string","6")` | | "a stri" |
| H824 | `LEFT(E12, E17)` | E12 = "non-numeric", E17 = `="2.99999"` (text) | "no" |
| K824 | `LEFT("a string", TRUE)` | | "a" |
| E1168 | `RIGHT("a string","6")` | | "string" |
| H1168 | `RIGHT(E11, E17)` | E11 = " beginsWithSpace", E17 as above | "ce" |
| K1168 | `RIGHT("a string", TRUE)` | | "g" |

H824/H1168 show numeric text is then truncated like a number ("2.99999" counts as 2).

Root cause: `text_num_arg` in `crates/rustsheet-engine/src/functions/text/common.rs` returns
`Error::VALUE` for `ArrayNode::Boolean` and `ArrayNode::String`. It is the one helper LEFT, RIGHT and
MID use for their count arguments. Coerce strings through the shared text-to-number parser
(`parse_excel_number` / the existing cast helper; do not write a new parser) and booleans to 1/0.

The same fix should also clear two gen-m2 cells that T-0219 lists under "booleans" (argument from a
reference holding a boolean): `gen-m2/ef5c55a81783b7d5` Calc!A40 (`MID(...,Data!B$4,5)`, B4 = TRUE,
expected "0.1x~") and `gen-m2/d04d501154afc771` Calc!A19 (`RIGHT(Data!A11,Data!B1)`, B1 = FALSE).
T-0219 keeps the huge-value and near-integer parts.

Rule recorded in docs/excel-semantics/coercion.md, "LEFT / RIGHT / MID numeric arguments" (MID numeric
text is UNVERIFIED there: add a probe cell or leave MID unchanged).

## Acceptance criteria
- [ ] The 6 lines `F	poi/FormulaEvalTestData_Copy.xlsx	EverythingTests!E824`, H824, K824, E1168,
      H1168 and K1168 disappear from eval/baseline/poi.txt with no new F lines in any suite.
- [ ] gen-m2/ef5c55a81783b7d5 Calc!A40 and gen-m2/d04d501154afc771 Calc!A19 leave
      eval/baseline/gen-m2.txt too, or the Outcome says which other cause keeps them (T-0219).
- [ ] Unit tests for LEFT, RIGHT and MID with a numeric-text count, a fractional numeric-text count,
      TRUE/FALSE, and non-numeric text (`#VALUE!`), scalar and array forms.
- [ ] The coercion.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Huge start_num/num_chars and near-integer counts (T-0219).
- UTF-16 code-unit counting (T-0249).
