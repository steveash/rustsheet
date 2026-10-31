---
id: T-2913
title: "Model::from_workbook refuses a workbook with a dangling index (style, shared string, shared formula, sheet, row, column) instead of panicking"
milestone: M12
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0026-python-legacy-surface.md
---

## Goal
`Model::from_workbook` (and so `Model::from_bytes`, `UserModel::from_bytes` and `rustsheet_xlsx::import::load_from_icalc`)
returns `Err` for a decoded `Workbook` whose indices dangle, instead of building a model that panics later.

## Context
SPEC-0026 decision 3 (docs/specs/SPEC-0026-python-legacy-surface.md). Directive 5: a decoded workbook is untrusted.
`Model::from_workbook` (crates/rustsheet-engine/src/model.rs) checks no index invariant. Reproduced by T-2882 on main at
7bd9090e (encode a one-cell workbook, change the bytes, decode, `Model::from_workbook`):
- cell style index 9999, then `set_user_input(0,1,1,"y")`: panics at styles.rs:478 (`style_is_quote_prefix` indexes
  `cell_xfs` unchecked);
- a `CellFormula { f: 77 }` with no shared formulas, then `evaluate()`: panics at evaluation.rs:819.
`get_style` (styles.rs) also indexes `fills`, `fonts` and `borders` unchecked.
Check, in one pass over the workbook, every index a later read trusts: cell, row and column style indices against
`cell_xfs`; each `cell_xfs` entry's font, fill, border and number-format ids; shared-string indices; shared-formula
indices per sheet; row and column ranges; sheet ids in defined names and the selected sheet; CF dxf ids; decoded counts and dimensions against the grid limits. List the
invariants in a doc comment on the validator, so the fuzz target (T-2895) finds what it misses.

## Acceptance criteria
- [x] The two reproductions above are unit tests that get `Err` from `Model::from_workbook` (no panic).
- [x] One test per other invariant checked (a dangling font, fill, border, shared-string, sheet index...).
- [x] Every workbook the engine writes or imports still loads: the xlsx/icalc round-trip tests, the xlsx test corpus
      (crates/rustsheet-xlsx/tests) and the eval suites are unchanged.
- [x] Decoded counts and dimensions past the grid limits are refused before anything is allocated from them.
- [x] The check is linear in the workbook's size.
- [x] `make check` green; reviewers approve.

## Out of scope
- Diffs (`apply_external_diffs`, T-2896).
- Changes to the xlsx importer itself. Its workbooks pass through the same check (`rustsheet_xlsx::import` ends in
  `Model::from_workbook`), so a check failure there surfaces as the importer's typed `XlsxError::Workbook`.

## Outcome
`crate::workbook_check::check_indices` runs first in `Model::from_workbook_with_clock`, before anything
is rebuilt or parsed from the workbook, and `SheetData::check_layout` checks the decoded cell storage
(chunk count and length, row and column bounds and order, the row counts). Both are one linear pass. The
invariants are listed in `check_indices`'s doc comment for T-2895's fuzz target. Unit tests:
`test/test_workbook_check.rs` (T-2882's two reproductions, which panic at styles.rs:478 and
evaluation.rs:833 without the check, and one test per invariant) and `sheet_data.rs`
(`check_layout_*`). Indices read through a tolerant lookup (CF dxf ids, a named style's `xf_id`, a
defined name's sheet id) are not checked, so workbooks that load today keep loading. One existing test
changed meaning: `malformed_import`'s empty-`<fonts>` workbook used to load, but its style 0 points at
a missing font, so the first style read would panic. It is now
`empty_fonts_with_an_xf_pointing_at_a_font_is_an_error`. DEVIATION entry: `docs/excel-semantics/xlsx-storage.md`
("A workbook whose indices dangle is refused outright").
