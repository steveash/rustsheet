---
id: T-0113
title: "API v0: ErrorKind/XlError conversions and engine error classification"
milestone: M1
lane: api
status: done
priority: P1
size: S
depends_on: [T-0100]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Provide the typed error layer of the facade so T-0102 and T-0103 never expose engine `String` errors.

## Context
SPEC-0001 "Values and errors" and "Errors". Engine `token::Error` has 12 variants; the harness currently stringifies them (`crates/rustsheet-eval/src/engine.rs`).

## Acceptance criteria
- [ ] `ErrorKind`, `XlError`, `RefError`, `EditError`, `LoadError`, `SaveError` in crate `rustsheet`, documented, `thiserror`-based.
- [ ] Test: every engine error variant converts to an `ErrorKind` and round-trips through its Excel literal.
- [ ] Engine `String` errors from sheet lookup/bounds are classified into `RefError` variants (tested).
- [ ] `make check` green; reviewers approve.

## Out of scope
Workbook methods (T-0102), edits (T-0103).

## Outcome
- New crate `rustsheet` (`crates/rustsheet`) with `ErrorKind`, `XlError`, `RefError`, `EditError`, `LoadError`, `SaveError`.
- `Error::error_type_code` is now the single ERROR.TYPE table in the engine; `ErrorKind::code` and `ERROR.TYPE` both use it.
- Deliberate, tested deviations from SPEC-0001: `ErrorKind::literal` is a static mirror of the engine's literals (guarded by `literals_match_engine`); engine `#ERROR!` maps to `Unknown`; `EditError::InvalidInput` has no `cell` yet (needs `CellRef`, T-0102); literal parsing is `ErrorKind::from_literal` (no `FromStr`).
- Engine `Display` prints `#N/IMPL` without `!` (its parser requires `!`); left as is.
- Eval: 0 regressions, 0 fixed.
