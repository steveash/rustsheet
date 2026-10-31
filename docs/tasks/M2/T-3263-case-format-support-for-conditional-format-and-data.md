---
id: T-3263
title: "Case-format support for conditional-format and data-validation formulas, then probe the 65-call nesting cap in them"
milestone: M2
lane: harness
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Oracle evidence for whether a conditional-format or data-validation formula nesting more than 65 calls makes
Excel refuse the workbook, and the importer matching it.

## Context
- T-3143 probed the 65-call cap for `@`/`#` and defined-name bodies (docs/excel-semantics/syntax.md "Formula
  nesting depth cap") but could not probe conditional formats or data validation: the `rustsheet-case/v1` format
  (crates/rustsheet-eval/src/case.rs, `deny_unknown_fields`) and tools/src/rustsheet_tools/spec2xlsx.py have no
  field for them. XlsxWriter writes both (`conditional_format` type `formula`, `data_validation` `custom`).
- The importer reads conditional formats (crates/rustsheet-xlsx/src/import/conditional_formatting.rs) but not data
  validation at all.
- Probe at 65 and 66 nested ABS calls for each; refused inputs are dropped after the run (T-1091, T-3097).

## Acceptance criteria
- [ ] Per-sheet case fields for a formula conditional format and a custom data validation, written by spec2xlsx
      (tested) and accepted (ignored or applied) by the engine adapter; documented in the case-format docs.
- [ ] excel-graph probes at 65 and 66 calls for both; syntax.md records the result.
- [ ] Where Excel refuses the workbook, the importer does too, with tests at 65 and 66.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Evaluating data validation.
