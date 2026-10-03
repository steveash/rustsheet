---
id: T-0115
title: "API v0: typed Value, Array and engine cell/value conversion"
milestone: M1
lane: api
status: in-progress
priority: P1
size: S
depends_on: [T-0113]
spec: docs/specs/SPEC-0001-public-api-v0.md
---

## Goal
Add `Value` (`Empty | Number | Text(Arc<str>) | Bool | Error(XlError)`, `#[non_exhaustive]`) and a dense row-major `Array` to crate `rustsheet`, plus crate-private conversion from the engine's `Cell`/`CellValue` variants, so T-0102 only adds the `Workbook` plumbing.

## Context
SPEC-0001 "Values and errors". `ErrorKind`/`XlError` already exist (T-0113). The eval harness's `crates/rustsheet-eval/src/engine.rs` currently extracts values from `Cell` by hand; the conversion written here is the single definition T-0102 later switches that adapter to. Conversion takes engine cell types by reference and does not need a `Model`, so it is independent of T-0101.

## Acceptance criteria
- [ ] `Value` and `Array` (`rows()`, `cols()`, `get(r, c)`, checked constructor that rejects `rows * cols` overflow) documented; `Value` implements `From<f64|bool|&str>`.
- [ ] Crate-private `Value::from_cell(..)` covers every engine `Cell` variant (empty, number, boolean, string, shared string, error, formula result of each type); a test enumerates all variants so a new engine variant fails to compile or fails the test.
- [ ] Error cells keep their literal and `FormulaValue::Error.message` becomes `XlError.detail`; test for `#DIV/0!`, `#N/A`, `#SPILL!`, `#CALC!`.
- [ ] Number equality in `PartialEq` is the derived `f64` one; documented that bitwise comparison for `ChangeSet` is T-0103's job.
- [ ] `make check` green; reviewers approve.

## Out of scope
Workbook reads, ref parsing (T-0114), switching the eval adapter (T-0102).
