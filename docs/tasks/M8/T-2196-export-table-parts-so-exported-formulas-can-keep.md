---
id: T-2196
title: "Export table parts so exported formulas can keep structured references (SPEC-0016 step 3)"
milestone: M8
lane: engine
status: in-progress
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
