---
id: T-3263
title: "Case-format support for conditional-format and data-validation formulas, then probe the 65-call nesting cap in them"
milestone: M2
lane: harness
status: done
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
- [x] Per-sheet case fields for a formula conditional format and a custom data validation, written by spec2xlsx
      (tested) and accepted (ignored or applied) by the engine adapter; documented in the case-format docs.
- [x] excel-graph probes at 65 and 66 calls for both; syntax.md records the result.
- [x] Where Excel refuses the workbook, the importer does too, with tests at 65 and 66.
- [x] `make check` green; reviewers approve.

## Out of scope
- Evaluating data validation.

## Outcome
Case format: per-sheet `conditional_formats` and `data_validations` (`{"ref", "formula"}`), written by
spec2xlsx as `<cfRule type="expression">` and `<dataValidation type="custom">` with prefixed formulas
(tested), accepted and ignored by the engine adapter, documented in docs/architecture/eval-harness.md.
excel-graph (2026-10-31): 65 nested ABS calls load in both and 66 make Excel refuse the workbook
(createSession 501; the 66-call cases were dropped after the run); a 299-character formula with no call
loads in both, so rule formulas have no 255-character file limit. syntax.md "Formula nesting depth cap"
records it. The importer refuses the same workbooks (`check_rule_formulas`; tests at 65 and 66), and the
cell, name-body and rule checks share `refuse_too_many_calls`. Eval delta: +4 probe cases, 4 cells, all
bit-exact.
