---
id: T-1011
title: "xlsx export and eval scoring use Error::stored_literal; import keeps rich-value kinds (SPEC-0003 X1)"
milestone: M1
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-1010]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
Error cells are saved and scored the way Excel stores them: `#GETTING_DATA` as `#N/A`, and the rich kinds by name. Cached `#BLOCKED!`, `#CONNECT!` and `#FIELD!` import as those kinds instead of `#VALUE!`.

## Context
SPEC-0003 "Design", sections "xlsx" and "Harness" (rollout X1). Evidence: probes/error-literals-new S!A1 stores `#N/A` (syntax.md "Newer error literals").
Code: `crates/rustsheet-xlsx/src/export/worksheets.rs` (the three `t="e"` writers), `import/worksheets.rs` (the rich-error guard); T-1010 left a `TODO(T-1011)` in `cached_error_name` that keeps cached `#CONNECT!`/`#BLOCKED!`/`#FIELD!` as `#VALUE!` until the exporter fallback lands, so remove it in this task. Since T-1010, a plain `t="e"` cell holding `#BLOCKED!`/`#CONNECT!`/`#FIELD!` imports as that kind (it was `#ERROR!`), so the exporter fallback covers those cells too, and the error comparison in `crates/rustsheet-eval`.

## Acceptance criteria
- [x] The exporter writes every error cell with `Error::stored_literal()` (`#N/A` for `#GETTING_DATA`). One exporter-only helper in rustsheet-xlsx writes `#SPILL!`, `#CALC!`, `#CONNECT!`, `#BLOCKED!` and `#FIELD!` as plain `#VALUE!` until T-1014, because Excel rejects those stored literals (T-0999). Tests cover both rules, and the PR calls out that `#CALC!`/`#SPILL!` lose their kind on an xlsx round trip in the interim.
- [x] The eval harness compares engine errors through `Error::stored_literal()`, never through the exporter's `#VALUE!` fallback. No second mapping exists, as checked with `rg`.
- [x] The importer reads the committed rich-errors-* oracle workbooks as `#BLOCKED!`/`#CONNECT!`/`#FIELD!`, with tests. The "stay `#VALUE!`" wording in arrays.md and in `rich_errors.rs` now says "unmapped errorTypes".
- [x] The remaining `probes/error-literals-new` and `-new2` cells pass and are recorded with `make eval-baseline`, with no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Writing `vm` rich-value metadata (T-1014).

## Outcome
The exporter writes error cells through `exported_error_literal` (Excel's stored form, with `#SPILL!`, `#CALC!`, `#CONNECT!`, `#BLOCKED!`, `#FIELD!` as `#VALUE!` until T-1014; `#CALC!`/`#SPILL!` therefore lose their kind on an xlsx round trip in the interim). The eval harness scores through the new `ErrorKind::stored_literal` (`#GETTING_DATA` -> `#N/A`), not the exporter fallback. The importer's `cached_error_name` keeps `#CONNECT!`/`#BLOCKED!`/`#FIELD!` (the `TODO(T-1011)` is gone). No committed xlsx workbook caches those three kinds (the rich-errors-* probes are oracle JSON sidecars), so the importer rule is covered by `cached_error_name` unit tests, not a workbook test. Eval: probes known failures 269 -> 260 (9 fixed, 0 regressions); `error-literals-new` and `-new2` fully pass.
