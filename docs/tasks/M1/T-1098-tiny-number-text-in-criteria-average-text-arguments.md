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
Tiny number text (`"1e-310"`, `"1e-309"`, `"0e309"`) reads the way Excel reads it in every
text-to-number path, not only the three T-0154 had evidence for.

## Context
T-0154 put the small-magnitude rule (numbers.md "Number text of 1E+308 or more is not a
number": nonzero below 1E-309 is not a number, 1E-309 up to DBL_MIN reads as 0, zero mantissas
need exponent less fraction digits within ±308) in `parse_excel_number`
(crates/rustsheet-engine/src/number_format.rs). Evidence covers `cast_number`, VALUE and the lexer
(probes/text-number-tiny, text-number-tiny-zero, text-number-words, literal-number-tiny). Other
callers now get the rule unprobed: criteria operands (`parse_criterion_number`, criteria.rs),
AVERAGEA text arguments (count_and_average.rs), NUMBERVALUE (string_format.rs) and formatted
number text (formatter/format.rs). `parse_finite_number` callers keep the old reading (`"1e-400"`
is 0, subnormals kept): TEXT format conditions (formatter/lexer.rs) and conditional formats
(conditional_formatting.rs); see TODO(T-1098) in crates/rustsheet-engine/src/test/test_nan_inf.rs.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers `COUNTIF(r,"1e-310")`, `COUNTIF(r,">1e-310")`, `COUNTIF(r,"<1e-309")`, `AVERAGEA` with `"1e-310"` and `"1e-309"` text arguments, `NUMBERVALUE("1e-310")`, `NUMBERVALUE("0e309")`, `="1,000e-312"+0` and a `TEXT` format condition `[>1e-310]`.
- [ ] Each path follows the evidence, reusing the T-0154 rule in number_format.rs (no copy); numbers.md records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
