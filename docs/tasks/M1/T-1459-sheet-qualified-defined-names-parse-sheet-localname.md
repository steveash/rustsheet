---
id: T-1459
title: "Sheet-qualified defined names parse: Sheet!LocalName"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A defined name written with a sheet prefix of this workbook, `=Sheet!LocalName` (the name scoped to
`Sheet`), parses and evaluates to that name, as Excel does.

## Context
Found in T-1382. poi/56737.xlsx Uses!D2 `=Defines!NR_To_A1` (Excel-cached "Test A1"; the name
`NR_To_A1` is scoped to sheet Defines) is `#ERROR!` today: after `Sheet!` the lexer only reads a
cell or range (`consume_range` in `crates/rustsheet-engine/src/expressions/lexer/mod.rs`), so a name
is an illegal token. The `[n]Sheet!Name` form already lexes as one identifier
(`consume_external_sheet_reference`, T-1382) the parser keeps it whole (`name_operand` in
`parser/operands.rs`) and `evaluate_external_name` splits it at evaluation; `[0]Sheet!Name` resolves the sheet-scoped name in `external_links.rs`
(`evaluate_own_name`). Reuse that path: lex `Sheet!Name` as an identifier when no cell follows and
resolve it to the `DefinedNameKind` of that sheet's scope. The formula text must round-trip as typed.
docs/excel-semantics/references.md "Cached values of a linked workbook" notes the gap.

## Acceptance criteria
- [ ] poi/56737.xlsx Uses!D2 leaves eval/baseline/poi.txt (baseline updated), no new F lines.
- [ ] Unit test: `=Sheet2!Local` reads the Sheet2-scoped name from a formula on Sheet1, and the
      formula text round-trips; `Sheet1!A1` still lexes as a reference.
- [ ] `[0]!Name` and `[0]Sheet!Name` resolve the same way at parse time (T-1382 keeps the prefix
      in the `DefinedNameKind` name and strips it only in evaluation, so ISREF, the reference
      context and a rename of the name miss it); unit test `ISREF([0]!CellName)` and a rename.
- [ ] references.md updated (drop the "does not parse yet" note).
- [ ] `make check` green; reviewers approve.

## Out of scope
- A sheet-qualified workbook-scoped name (`Sheet!GlobalName`): unprobed, leave as is.
