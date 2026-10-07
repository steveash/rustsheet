---
id: T-1101
title: "Number text longer than 255 characters is #VALUE!"
milestone: M1
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Text of more than 255 characters read as a number (`0+A` and `VALUE` are probed; probe any other text-to-number path before changing it) is `#VALUE!`, as in Excel.

## Context
Found by T-1097: docs/excel-semantics/numbers.md "Number text of 1E+308 or more is not a number" (255-character rule). probes/text-number-long-zero fails today because the engine reads long text as a number. Code: `parse_excel_number` in crates/rustsheet-engine/src/number_format.rs and the callers that coerce text. The engine already has this limit for clock text (`MAX_CLOCK_TEXT_LEN` in functions/date_and_time/time_text.rs and a `len > 255` check in expressions/utils/mod.rs); share one constant, do not add a second (see T-0273). Check whether a shared entry point exists before adding the length test in one place only.

## Acceptance criteria
- [x] probes/text-number-long-zero passes in full (record fixes with `make eval-baseline`); a unit test covers LEN 255 (number) and 256 (`#VALUE!`).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Number text over 255 characters (spaces counted) is `#VALUE!` in number coercion and VALUE; one shared `MAX_NUMBER_TEXT_LEN` (cast.rs) now also serves clock text. probes/text-number-long-zero passes in full (138 cells fixed). Not changed: criteria, AVERAGE/SUM text arguments and NUMBERVALUE are unprobed for length (still no limit).
