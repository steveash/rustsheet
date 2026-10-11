---
id: T-1093
title: "Decide whether to refuse formulas over 8,192 characters on load (Excel refuses the workbook)"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Decide, with a decision record if it is a DEVIATION, whether RustSheet should refuse a workbook
formula longer than 8,192 characters on load, as Excel does.

## Context
T-1091 found that Excel for the web makes a workbook unloadable (`FileCorruptTryRepair`) when a
formula holds more than 8,192 stored characters (without the leading `=`). RustSheet has no formula
text length limit; it only caps value operators at `MAX_VALUE_OPERATORS` (4,096). See
docs/excel-semantics/syntax.md ("Formula nesting depth cap"); code: crates/rustsheet-engine/src/expressions/parser/depth.rs.

## Acceptance criteria
- [x] A recorded decision (implement the refusal, or accept the divergence with a DEVIATION entry).
- [x] If implemented: a unit test at 8,192 and 8,193 stored characters; `syntax.md` updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Typing a long formula through the API (not probed).

## Outcome
Implemented the refusal, recorded in decision 0047.
- **What changed.** The xlsx importer now refuses a workbook (`XlsxError::Workbook`) that holds a stored
  formula (`<f>` text) of more than 8,192 characters. That is what Excel for the web does (T-1091,
  T-1060), so it is not a deviation.
- **Why the importer.** A cell-level parse error was rejected because it would not survive the importer's
  A1 to R1C1 storage.
- **Tests.** The new import test `formula_longer_than_8192_characters_refuses_the_workbook` checks that
  8,192 characters load and 8,193 refuse.
- **Docs.** The rule is in docs/excel-semantics/syntax.md, "Formulas over 8,192 characters refuse the
  workbook".
- **Existing workbooks.** No upstream or eval workbook holds a formula longer than 393 characters.
- **Eval delta.** None.
- **Follow-up.** T-1386 probes typed formulas, defined names and UTF-16 counting.
