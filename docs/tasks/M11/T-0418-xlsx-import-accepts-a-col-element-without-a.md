---
id: T-0418
title: "xlsx import accepts a col element without a width attribute (port ironcalc 9d5fa0dc)"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A workbook whose `<cols>` has a `<col>` with no `width` attribute imports, and that column takes the sheet's default width, instead of the whole import failing.

## Context
Upstream IronCalc 9d5fa0dc (2026-10-12, listed by the 2026-10-13 steward run) fixed this in
`xlsx/src/import/worksheets.rs`. `width` is optional in ECMA-376 Part 1, 18.3.1.13. Tools that only
set a style or hide columns leave it out. Our `load_columns`
(`crates/rustsheet-xlsx/src/import/worksheets.rs`, `get_attribute(col, "width")?`) still rejects such
a file. Upstream falls back to `<sheetFormatPr defaultColWidth>`, then 8.43. Before reusing that
constant, check whether we already have a default-column-width constant (`rg -n "8.43|default_column_width|DEFAULT_COLUMN_WIDTH" crates`).
Port with `Ported-From: ironcalc@9d5fa0dc1d596fb9ace1f9f5491df34dc051ae50`. Upstream is not
evidence of Excel values, but this is import robustness, not a computed value.

## Acceptance criteria
- [ ] A unit test imports a sheet with `<col min=".." max=".." style=".."/>` (no width) and with `hidden="1"` and no width; both load, and the column gets the sheet default width.
- [ ] A missing `sheetFormatPr`/`defaultColWidth` falls back to the existing default, with no new copy of the constant.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column width round-trip precision on export.

## Outcome
`load_columns` no longer requires `width`: a `<col>` without it takes `<sheetFormatPr defaultColWidth>`
when present, else `DEFAULT_COLUMN_WIDTH / COLUMN_WIDTH_FACTOR` (the engine constant, now exported
from `rustsheet_engine`). `load_columns` moved to `import/columns.rs` so `worksheets.rs` (oversized, ratcheted) shrinks. Tests in `crates/rustsheet-xlsx/tests/col_without_width.rs`. Eval delta: none.
Ported-From: ironcalc@9d5fa0dc1d596fb9ace1f9f5491df34dc051ae50
