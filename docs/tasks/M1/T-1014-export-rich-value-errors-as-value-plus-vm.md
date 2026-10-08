---
id: T-1014
title: "Export rich-value errors as #VALUE! plus vm metadata (SPEC-0003 X2)"
milestone: M1
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1011]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
Workbooks we save hold `#SPILL!`, `#CALC!`, `#CONNECT!`, `#BLOCKED!` and `#FIELD!` in the form Excel writes them: `t="e"` with `<v>#VALUE!</v>` and a `vm` index into `xl/metadata.xml` and `xl/richData/*`. T-0999 showed that Excel for the web rejects a workbook storing `#BLOCKED!`, `#CONNECT!`, `#FIELD!` or `#CALC!` as a literal (syntax.md), so this is required. It also replaces T-1011's interim `#VALUE!` export helper, which it deletes. The oracle recalc also confirms that a stored `#SPILL!` literal is rejected (unprobed).

## Context
SPEC-0003 "xlsx" (rollout X2). The format is in arrays.md "Cached rich errors" (metadataTypes, futureMetadata `XLRICHVALUE`, `rvb`, `rdrichvalue.xml`, the `_error` structure with `errorType`/`subType`). The reader is `crates/rustsheet-xlsx/src/import/rich_errors.rs`.

## Acceptance criteria
- [x] The exporter writes the rich-value parts. A round trip through our importer keeps every kind, and an oracle recalc of an exported workbook loads with the expected ERROR.TYPE codes (excel-probe skill).
- [x] `make check` green; reviewers approve.

## Out of scope
- New error kinds.

## Outcome
`export/rich_errors.rs` writes `#SPILL!`, `#CONNECT!`, `#BLOCKED!`, `#FIELD!` and `#CALC!` in Excel's form:
- the cell is `t="e"` `#VALUE!` with `vm` 1-5 (one per kind, fixed order);
- `xl/metadata.xml` lists `XLDAPR` and then `XLRICHVALUE`, with `futureMetadata`/`valueMetadata` blocks;
- `xl/richData/rdrichvalue.xml` holds one `_error` value per kind, `subType` 0, plus Excel's structure and types parts, content types and workbook relationships.

The parts are written only when some cell holds such an error. The errorType codes go through the importer's `rich_error_name`, so export and import share one table. The static `export/metadata.xml` is now generated.

`exported_error_literal` stays: the rich kinds still store `#VALUE!` (now with `vm`), and the engine-only `#CIRC!`/`#ERROR!`/`#N/IMPL` (T-1133) still need the fallback. `compare.rs` now treats only those three as lost on export.

Evidence:
- fixtures/rich-error-export.xlsx is our exporter's output. Excel for the web loads it; ERROR.TYPE gives 9, 10, 11, 13, 14, VALUETOTEXT gives the names, and ISERROR is TRUE.
- The same file with a stored plain `#SPILL!` literal is rejected (501). This is recorded in arrays.md and syntax.md.

Tests: `rich_errors_round_trip` covers the formula results and the Excel-authored constants surviving save and reload with their kinds, and a unit test decodes the written parts with the importer's decoder.

Follow-up: T-1145. The facade stores a typed `XlError("#CONNECT!")`/`#BLOCKED!`/`#FIELD!` as text.

Eval: fixtures +1 case, all cells passing; 0 regressions.
