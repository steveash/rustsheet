---
id: T-1093
title: "Decide whether to refuse formulas over 8,192 characters on load (Excel refuses the workbook)"
milestone: M1
lane: engine
status: in-progress
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
- [ ] A recorded decision (implement the refusal, or accept the divergence with a DEVIATION entry).
- [ ] If implemented: a unit test at 8,192 and 8,193 stored characters; `syntax.md` updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typing a long formula through the API (not probed).
