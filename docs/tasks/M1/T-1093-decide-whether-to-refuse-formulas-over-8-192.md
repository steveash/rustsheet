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
- [x] (n/a, not implemented) If implemented: a unit test at 8,192 and 8,193 stored characters; `syntax.md` updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Typing a long formula through the API (not probed).

## Outcome
Decided not to refuse, in decision 0047. RustSheet keeps loading formulas over 8,192 stored characters,
which Excel for the web refuses at the workbook level.
- **Why not refuse.** No Excel value can exist for such a cell, so there is nothing to score against.
  Directive 5 is already met by the operator and depth caps and by `ImportLimits`. A length refusal would
  also not survive the importer's A1 to R1C1 storage: the stored text is parsed again in R1C1 mode, where
  a long chain parses cleanly and the text is longer than the A1 form.
- **Where it is recorded.** The rule is in docs/excel-semantics/syntax.md as "Formulas over 8,192
  characters load", with `Status: DEVIATION (decision 0047)`.
- **Code and eval.** No code change. Eval delta: none.
