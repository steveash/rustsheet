---
id: T-1089
title: "Load bare newer function names in xlsx as #NAME? and export eta arguments as _xleta."
milestone: M11
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
Workbooks round-trip function names as Excel stores them: a newer function stored without its
`_xlfn.` prefix evaluates to `#NAME?` on import, and an eta-reduced function argument
(`GROUPBY(..., SUM)`) is exported as `_xleta.SUM`.

## Context
docs/excel-semantics/xlsx-storage.md, "Newer functions need `_xlfn.` in the file, eta arguments
`_xleta.`" (probes/catalog/function-name-forms A1-A6, function-name-forms-2 A1-A3; T-0040).
- Import: the parser strips `_xlfn.`/`_xlfn._xlws.` and then looks the name up, so a bare
  `PERCENTOF(1,2)` read from a file resolves to the function
  (crates/rustsheet-engine/src/expressions/parser/mod.rs, around lines 712-787). In Excel it is
  `#NAME?`. The same parser reads typed formulas, where the bare name is right, so the rule must
  apply only to formulas read from xlsx. That needs a parse mode or an import-time check against
  the list of names that need a prefix (`Function::to_xlsx_string`).
- Export: `Function::to_xlsx_string` writes `_xlfn.` names, but nothing writes `_xleta.` for a
  function passed by name. Excel reads such a bare name as `#NAME?`.
Tests need xlsx inputs (an import test with a bare `_xlfn`-less newer name) and an export test.
- corpus-bench/1726 `July-KPIs!A1` `=IMAGE("https://...")` stored without `_xlfn.`: Excel `#NAME?`, engine `#BLOCKED!` (T-0427 run; the case's other 33 cells are T-1770).

## Acceptance criteria
- [ ] Importing a workbook whose `<f>` is `PERCENTOF(1,2)` gives `#NAME?`; `_xlfn.PERCENTOF(1,2)`
  gives 0.5; typed formulas are unchanged.
- [ ] Exporting `=GROUPBY(A1:A2,B1:B2,SUM)` writes `_xleta.SUM`, and re-importing evaluates the same.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing GROUPBY/PIVOTBY.
