---
id: T-0113
title: "API v0: ErrorKind/XlError conversions and engine error classification"
milestone: M1
lane: api
status: in-progress
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
