---
id: T-1098
title: "Tiny number text in criteria, AVERAGE text arguments, NUMBERVALUE and format conditions"
milestone: M1
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Tiny number text (`"1e-310"`, `"1e-309"`, `"0e309"`) reads the way Excel reads it in the paths that use `parse_finite_number`: COUNTIF/SUMIF criteria operands, AVERAGE/COUNT text arguments, NUMBERVALUE, and TEXT format conditions.

## Context
T-0154 applied the small-magnitude rule (numbers.md "Number text of 1E+308 or more is not a
number": nonzero below 1E-309 is not a number, 1E-309 up to DBL_MIN reads as 0, zero mantissas
need exponent less fraction digits within ±308) to `parse_excel_number` only (`cast_number`,
VALUE, the lexer), where probes/text-number-tiny, text-number-tiny-zero, text-number-words and
literal-number-tiny give evidence. `parse_finite_number` (crates/rustsheet-engine/src/number_format.rs)
still reads `"1e-400"` as 0 and keeps subnormals; no probe covers its callers yet
(TODO(T-1098) in crates/rustsheet-engine/src/test/test_nan_inf.rs).

## Acceptance criteria
- [ ] A probe case (excel-graph) covers `COUNTIF(r,"1e-310")`, `COUNTIF(r,">1e-310")`, `COUNT("1e-310",1)`, `AVERAGE("1e-309",2)`-style text arguments, `NUMBERVALUE("1e-310")`, `NUMBERVALUE("0e309")` and a `TEXT` format condition `[>1e-310]`.
- [ ] `parse_finite_number` follows the evidence (reusing the T-0154 helpers in number_format.rs, no copy); numbers.md records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
