---
id: T-2894
title: "Model::from_workbook refuses a workbook with a dangling index (style, shared string, shared formula, sheet, row, column) instead of panicking"
milestone: M12
lane: engine
status: todo
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
indices per sheet; row and column ranges; sheet ids in defined names and the selected sheet; CF dxf ids. List the
invariants in a doc comment on the validator, so the fuzz target (T-2895) finds what it misses.

## Acceptance criteria
- [ ] The two reproductions above are unit tests that get `Err` from `Model::from_workbook` (no panic).
- [ ] One test per other invariant checked (a dangling font, fill, border, shared-string, sheet index...).
- [ ] Every workbook the engine writes still loads: the xlsx/icalc round-trip tests and the eval suites are unchanged.
- [ ] The check is linear in the workbook's size.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Diffs (`apply_external_diffs`, T-2896).
- The xlsx importer, which builds its own workbook (fuzzed by fuzz/fuzz_targets/xlsx_import.rs).
