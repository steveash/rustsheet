---
id: T-0257
title: "LEFT/RIGHT/MID: numeric text and boolean count arguments coerce to numbers (6 poi + 2 gen-m2 known failures)"
milestone: M2
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
LEFT and RIGHT `num_chars` and MID `start_num`/`num_chars` accept numeric text and booleans the way
Excel does: `"6"` counts as 6, a text `"2.99999"` as 2 (truncated like the number), TRUE as 1 and
FALSE as 0. Today any text or boolean count is `#VALUE!`.

## Context
Found by the gap triager on 2026-10-06. Semantics: docs/excel-semantics/coercion.md "LEFT / RIGHT /
MID numeric arguments" (VERIFIED). Failing cells (expected / actual):
- poi/FormulaEvalTestData_Copy.xlsx (Excel-cached, Excel 15) EverythingTests:
  E824 `LEFT("a string","6")` "a stri" / `#VALUE!`; H824 `LEFT(E12, E17)` (E12 "non-numeric", E17
  `="2.99999"`) "no" / `#VALUE!`; K824 `LEFT("a string", TRUE)` "a" / `#VALUE!`;
  E1168 `RIGHT("a string","6")` "string" / `#VALUE!`; H1168 `RIGHT(E11, E17)` (E11
  " beginsWithSpace") "ce" / `#VALUE!`; K1168 `RIGHT("a string", TRUE)` "g" / `#VALUE!`.
- gen-m2 (excel-graph): `gen-m2/ef5c55a81783b7d5` Calc!A40 `=MID((Data!E$4&(Data!A10&"x~*")),Data!B$4,5)`
  (B4 = TRUE): "0.1x~" / `#VALUE!`; `gen-m2/d04d501154afc771` Calc!A19
  `UPPER(RIGHT(Data!A11,Data!B1))>=...` (B1 = FALSE, so RIGHT gives ""): TRUE / `#VALUE!`.
  These two were listed in T-0219; this task takes them over.
Passing neighbours that must stay green: D824/D1168 (count 0), G824 `LEFT("a string",B9)` "a ",
I824 `LEFT(TRUE,2)` "TR", J824 `LEFT(G7:H9,H10)` `#VALUE!`.

Code: `text_num_arg` in `crates/rustsheet-engine/src/functions/text/common.rs` rejects
`ArrayNode::Boolean` and `ArrayNode::String` (its doc comment states the old, wrong rule). Reuse the
existing coercion `Model::array_node_to_number` in `crates/rustsheet-engine/src/cast.rs` (REPLACE
already uses it in `string_format.rs`) instead of writing a new one: it maps booleans to 1/0, empty
to 0, numeric text through `cast_number`, and other text to `#VALUE!`. `left_element`,
`right_element` and `mid_element` then need `&self` access (or the helper result passed in); keep
the negative-count and start < 1 checks after coercion.

## Acceptance criteria
- [ ] These 8 lines disappear from the baselines (`make eval-baseline`), with no new F lines:
      eval/baseline/poi.txt `poi/FormulaEvalTestData_Copy.xlsx` EverythingTests!E824, H824, K824,
      E1168, H1168, K1168; eval/baseline/gen-m2.txt `gen-m2/ef5c55a81783b7d5.case.json` Calc!A40
      and `gen-m2/d04d501154afc771.case.json` Calc!A19.
- [ ] Unit tests: `LEFT("abc","2")` "ab", `RIGHT("abc",TRUE)` "c", `MID("abcdef","2","2.9")` "bc",
      `LEFT("abc",FALSE)` "", `LEFT("abc","x")` `#VALUE!`.
- [ ] The coercion.md entry gets an `Implemented in:` line naming the helper.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Huge counts (about 1E+15 is `#VALUE!`) and near-integer counts: T-0219.
- UTF-16 counting of LEFT/RIGHT/MID positions: T-0249.
- LEFTB/RIGHTB/MIDB.
