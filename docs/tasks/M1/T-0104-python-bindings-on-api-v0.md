---
id: T-0104
title: "Python bindings on API v0"
milestone: M1
lane: api
status: done
priority: P1
size: M
depends_on: [T-0103]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Expose the facade through PyO3 (`rustsheet.Workbook`, `open`, `edit` context manager, values, XlError values, typed exceptions), keeping the old Model bindings until parity.

## Context
bindings/python currently wraps Model/UserModel directly.

## Acceptance criteria
- [x] New pytest suite for the facade API; GIL released around recalc/load/save.
- [x] Docs/examples updated.
- [x] `make check` green; reviewers approve.

## Out of scope
numpy/arrow bulk (M12).

## Outcome
- New `bindings/python/src/facade.rs`: `Workbook` (open/from_bytes/save/read/`edit()`), `Transaction`, `XlError` (returned value, equal by kind), `Formula`, and `RustSheetError` with `RefError`/`EditError`/`LoadError`/`SaveError` subclasses. `rs.open` shorthand. Old `Model`/`UserModel` bindings are untouched until parity.
- Deliberate deviations from the spec text: `wb.changes` is a plain dict (`inputs`/`values`/`sheets`), not a typed class; `Transaction` buffers ops and applies them as one atomic `Workbook::edit` at `__exit__`, so op errors surface at the end of the `with` block (sheet ops address sheets by name, resolved with the facade's `resolve_sheet_by_name`); added `SaveError`, `from_bytes`, `to_xlsx_bytes`.
- GIL is released around load, save, recalc, edit commit and every workbook read; the mutex is only ever taken with the GIL released (review caught a GIL/mutex deadlock; pinned by a threaded test).
- Follow-ups: T-0931 (parser nesting-depth stack overflow found here). Observed, not filed: typing `12%` stores number format `#,##0%` where Excel gives `0%` (needs oracle evidence).
