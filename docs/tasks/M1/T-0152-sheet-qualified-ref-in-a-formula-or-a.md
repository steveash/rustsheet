---
id: T-0152
title: "Sheet-qualified #REF! in a formula or a defined name evaluates to #REF! (2 poi known failures)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A reference that Excel has already turned into `#REF!` but that keeps its sheet prefix
(`'1'!#REF!`, `Sheet1!#REF!`) evaluates to `#REF!`. This holds when it is the cell formula and
when it is the body of a defined name.

## Context
Rule and evidence: docs/excel-semantics/references.md, "A sheet-qualified #REF! is #REF!".
Excel writes these when a referenced range is deleted. Evidence, desktop Excel cached values
(`make eval-show CASE=...`):
- `poi/customIndexedColors.xlsx` (Excel 16.0) `_TGK_HIDDEN!B1` `='1'!#REF!`: expected `#REF!`,
  actual `#ERROR!` (ErrorMismatch). The formula does not parse. A spec case shows the same for
  `=Sheet1!#REF!` and `=SUM(Sheet1!#REF!)` (both `#ERROR!`), while a bare `=#REF!` already gives
  `#REF!`.
- `poi/49783.xlsx` (Excel 14.0) Sheet1!A2 `=sale_1*sale_2`, where the workbook defines
  `sale_1` and `sale_2` as `Sheet1!#REF!`: expected `#REF!`, actual `#NAME?` (Unsupported). In the
  same case A1 `=#REF!*#REF!` (`#REF!`) and B2 `=aaa*sale_1` (`#NAME?`, `aaa` is undefined) already
  pass and must keep passing.

Code paths:
- Formula parser and lexer: `crates/rustsheet-engine/src/expressions/parser/` (and `lexer/`). A
  sheet prefix followed by the `#REF!` error literal should parse to the `#REF!` error node.
- Defined names: `parse_defined_names` in `crates/rustsheet-engine/src/new_empty.rs` tries
  `ParsedReference::parse_reference_formula`, then the full parser, and anything that is not a
  LAMBDA becomes `ParsedDefinedName::InvalidDefinedNameFormula`. The `DefinedNameKind` arm in
  `crates/rustsheet-engine/src/model.rs` turns that into `#NAME?`. A name whose body is a
  (sheet-qualified) error literal should evaluate to that error. Extend the existing enum and
  match arms. Do not add a parallel lookup. `functions/information/mod.rs` matches the same enum
  (ISREF and friends), so keep it exhaustive.

## Acceptance criteria
- [ ] `F	poi/customIndexedColors.xlsx	_TGK_HIDDEN!B1` and `F	poi/49783.xlsx	Sheet1!A2`
      disappear from eval/baseline/poi.txt, with no new F lines in any suite.
- [ ] Unit tests: `='Sheet1'!#REF!`, `=Sheet1!#REF!` and `=SUM(Sheet1!#REF!)` give `#REF!`. A
      defined name with body `Sheet1!#REF!` used in `=name*1` gives `#REF!`.
      `=Sheet1!#REF!` and `=SUM(Sheet1!#REF!)` are hand expectations (no Excel cell yet; the
      references.md entry says so). If the implementation makes them differ from the quoted form,
      probe them first (`excel-probe` skill).
- [ ] references.md "A sheet-qualified #REF! is #REF!" gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Defined names whose body is a general formula such as `SUM(tblExpenses[Amount])`
  (poi/simple-monthly-budget.xlsx, poi/simple-table-named-range.xlsx, 6 cells). That is a
  bigger engine feature and needs its own opus task.
- External-workbook references (`[1]Sheet!A1`, T-0124).
- Keeping the sheet prefix when such a formula is exported back to xlsx (no evidence here).

## Outcome
The lexer reads `Sheet!#REF!` (quoted or not) as the `#REF!` error token; defined names whose body is an
error literal evaluate to that error (new `ParsedDefinedName::ErrorLiteral`; `new_defined_name` accepts
it). Eval: poi known failures 899 -> 897 (fixed 2: 49783.xlsx Sheet1!A2, customIndexedColors.xlsx
_TGK_HIDDEN!B1), 0 regressions. Other sheet-qualified errors (`Sheet1!#NAME?`) stay `#ERROR!`.
