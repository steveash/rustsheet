---
id: T-2196
title: "Export table parts so exported formulas can keep structured references (SPEC-0016 step 3)"
milestone: M8
lane: engine
status: done
priority: P2
size: M
tier: opus
spec: docs/specs/SPEC-0016-structured-references-in-the-parsed-formula.md
depends_on: [T-2195]
---

## Goal
An exported workbook carries its tables, and its formulas keep their structured references in the file form Excel accepts.

## Context
SPEC-0016 rollout step 3. The xlsx exporter writes no `<tableParts>` or `xl/tables/*.xml` (`rg "tablePart|xl/tables" crates/rustsheet-xlsx/src/export` finds nothing), so `to_excel_string` keeps printing lowered ranges after T-2195. File forms Excel requires: docs/excel-semantics/xlsx-storage.md "Structured references in the file" (a bare name as `Table[]`, `@` as `[#This Row]`, the escape before the special character, the totals-row metadata).

## Acceptance criteria
- [ ] The exporter writes each table's part and relationship, with the totals-row metadata xlsx-storage.md requires; an import of the export gives the same tables.
- [ ] `to_excel_string` prints the preserved structured reference, a bare name as `Table[]`.
- [ ] A workbook with tables and structured references round-trips (import, export, import) with the same formula text and values; use an Excel-authored workbook from rustsheet-xlsx `tests/calc_tests` if one has structured references.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Table styles beyond what import keeps; FORMULATEXT spelling (T-1683).

## Outcome
The xlsx export writes every table as a table part (`crates/rustsheet-xlsx/src/export/tables.rs`): its `<tablePart>`
in the sheet, the relationship after the sheet's external hyperlinks, the content type, and `xl/tables/tableN.xml`
with the totals-row metadata (a formula cell as `totalsRowFunction="custom"` + `<totalsRowFormula>`, a text cell as
`totalsRowLabel`) and an `<autoFilter>` over the header and data rows. `to_excel_string` now prints each resolved
structured reference from the preserved text (`StructuredReference::file_text`: a bare name as `Table[]`, `@` as
`[#This Row]`, the escape before the special character); an unresolved one is still `#REF!`.
The Excel-authored `tests/calc_test_no_export/tables.xlsx` moved to `tests/calc_test_tables/` with the save round
trip on (`test_tables_tables`: import, export, import, same values); unit tests in `export/test/test_tables.rs` pin
the file forms, the same tables after re-import, formula text and values. An applied filter keeps only its flag
(written as a `<filterColumn>` with no criterion): follow-up T-2838 keeps the criteria and probes Excel's reading.
Eval delta: none (the export is not on the eval path).
