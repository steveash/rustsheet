---
id: T-1412
title: "TEXTSPLIT with an empty row_delimiter is #VALUE! even when a column delimiter is given"
milestone: M7
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTSPLIT with an empty row_delimiter is `#VALUE!` even when a column delimiter is given, as in Excel.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/d244411dd90f7fbd Calc!A6 `=TEXTSPLIT(((Data!D8&Data!E$5)&("abc"&"x~*")),((Data!E8&Data!B$4)&VALUETOTEXT(Data!C8)),"")`:
  text `"TRUE"&"abc"&"x~*"`, col_delimiter "TRUE1E-15a*", row_delimiter "". Excel `#VALUE!`, engine the whole text (one cell).
Engine today: `TEXTSPLIT("abc","")` and `TEXTSPLIT("abc",,"")` are already `#VALUE!`, but `TEXTSPLIT("abc","b","")` is "a"
(the empty row delimiter is ignored). Before generalising, probe with the excel-probe skill `TEXTSPLIT("abc","b","")`,
`TEXTSPLIT("abc","","b")`, `TEXTSPLIT("abc",{"b",""})` and `TEXTSPLIT("abc","b",{",",""})` (an empty element inside a list).
Semantics: docs/excel-semantics/text.md "TEXTSPLIT with an empty delimiter is #VALUE!". Code: `fn_textsplit`
(crates/rustsheet-engine/src/functions/text/textsplit.rs). Delimiter ranges are T-1361.
T-1361 (probes/textbefore-delimiter-range A44, excel-graph 2026-10-18) settles the list form: `TEXTSPLIT("abc",",",A9:B9)` with A9
blank, B9 "b" is `#VALUE!` (the blank cell is an empty row delimiter; ours drops it via `skip_empty` in `split_delimiters` and gives
{"a";"c"}). That cell is a known failure in eval/baseline until this task lands.

## Acceptance criteria
- [ ] gen-m9/d244411dd90f7fbd A6 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] The empty-delimiter probe cells pass; text.md's entry is VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
