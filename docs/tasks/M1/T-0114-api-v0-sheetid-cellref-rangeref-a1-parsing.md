---
id: T-0114
title: "API v0: SheetId/CellRef/RangeRef types and A1 parsing/printing"
milestone: M1
lane: api
status: done
priority: P1
size: S
depends_on: [T-0100]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Add `refs.rs` to crate `rustsheet`: `SheetId`, `CellRef`, `RangeRef`, `Target`, and pure A1 parse/print functions that do not need a `Workbook` (sheet names are resolved through a caller-supplied lookup), so T-0102 only wires them to the model.

## Context
SPEC-0001 "Identities and references". Not blocked by T-0101 because nothing here touches `Model`. Reuse the engine's `LAST_ROW`/`LAST_COLUMN` and column-name helpers (search with `rg` first); do not re-implement column letter conversion.

## Acceptance criteria
- [ ] `SheetId`, `CellRef`, `RangeRef` (`RangeRef::new` returns `Result<_, RefError>`), `Target` with `From<CellRef|RangeRef|&str>`, `Display` printing A1 with sheet names quoted as needed; all public items documented.
- [ ] `parse_cell(a1, resolve_sheet)` / `parse_range(...)` accept `A1`, `$A$1`, `Sheet1!A1`, `'My Sheet'!A1:B2`, `'It''s'!A1`, `A:A`, `1:3`; unqualified refs resolve to the first sheet via the lookup; sheet lookup is case-insensitive.
- [ ] Defined names and structured references return `RefError::Unsupported`; out-of-bounds (row 1,048,577, column XFE) returns `RefError::OutOfBounds`; garbage returns `RefError::Invalid`. No panics (test with empty string, `!`, `A1:`, `:A1`, 10k-char input).
- [ ] Print/parse round-trip unit tests for quoted names, whole row/column and bounds corners (`XFD1048576`).
- [ ] No new duplicated column-name helper (`scripts/check_duplication.py` green); `make check` green; reviewers approve.

## Out of scope
`Workbook` methods (`cell_ref`, `value`; T-0102), `values()` size cap (T-0102).

## Outcome
`crates/rustsheet/src/refs.rs` adds `SheetId`, `CellRef`, `RangeRef`, `Target`, `parse_cell`/`parse_range`
and `resolve_sheet_by_name`. Column/quote helpers come from the engine (`column_to_number`,
`number_to_column`, `quote_name`); `LAST_ROW`/`LAST_COLUMN` are now re-exported from the engine crate.
`Display` omits the sheet (no name on `SheetId`); `to_a1_in(name)` prints the qualified form (SPEC-0001 updated).
