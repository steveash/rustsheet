---
id: T-1101
title: "Number text longer than 255 characters is #VALUE!"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Text of more than 255 characters read as a number (`0+A`, `VALUE`, other text-to-number paths) is `#VALUE!`, as in Excel.

## Context
Found by T-1097: docs/excel-semantics/numbers.md "Number text of 1E+308 or more is not a number" (255-character rule). probes/text-number-long-zero fails today because the engine reads long text as a number. Code: `parse_excel_number` in crates/rustsheet-engine/src/number_format.rs and the callers that coerce text. Check whether a shared entry point exists before adding the length test in one place only.

## Acceptance criteria
- [ ] probes/text-number-long-zero passes in full (record fixes with `make eval-baseline`); a unit test covers LEN 255 (number) and 256 (`#VALUE!`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
