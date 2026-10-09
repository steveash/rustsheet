---
id: T-0938
title: "TEXT General: 11-character fixed-versus-scientific choice for small numbers (2 known failures)"
milestone: M1
lane: engine
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`TEXT(x, "General")` picks fixed or scientific notation for small numbers the way Excel does. That
fixes the frozen M1 probe cell `probes/number-text-exponent-digits` Calc!D8 and the two poi cells
below.

## Context
The planner moved this task from M10/functions-text to M1/engine on 2026-10-09. The frozen M1 probe
list (eval/milestones/m1-probes.txt) has a known failure with this root cause, and T-0986 named this
task as its fix. The code is the General branch of `crates/rustsheet-engine/src/formatter/format.rs`.
That branch prints fixed (`{:.9}`, trailing zeros trimmed) whenever `1e-8 <= |x| < 1e11`, and
scientific otherwise. The window is wrong in both directions.

Evidence (excel-graph):
- `probes/number-text-exponent-digits` Calc!D8: `=TEXT(1.23456789012345E-5,"General")` expected
  "1.23457E-05", actual "0.000012346". Excel uses E notation even though a rounded fixed form fits
  in 11 characters.
- `poi/GeneralFormatTests.xlsx` (T-0019), `=TEXT(Cn, Bn)` with `Bn` = "General": A27 `1E-09` expected
  "0.000000001", actual "1E-09"; A61 `-1E-09` expected "-0.000000001", actual "-1E-09". Here Excel
  uses fixed even though the value is below 1e-8.
- Cells that bound the rule and pass today (they must keep passing): A28 `1E-10` = "1E-10", A26
  `0.01` = "0.01", A13 `1.2345678912` = "1.234567891", A17 `1E10` = "10000000000", A18 `1E11` =
  "1E+11", and number-text-exponent-digits D9:D15 plus probes/text-general-exponent-boundary.

Reading: General keeps fixed notation while the digits fit in 11 characters (the minus sign does not
count, A61), but not when fixed shows fewer significant digits than E notation would. D8's fixed
form "0.000012346" keeps 5 digits against 6 in "1.23457E-05". A13's fixed form keeps 10 digits
against 6. This rule is the planner's reading of the evidence, not a verified rule.

The implementer must check the rule against every TEXT/General cell in the eval suites. Ties (equal
digit counts, e.g. 0.000123456789) are unprobed. If the chosen rule decides a tie, file a probe task
(M2) and record the assumption.

This is the General *format*. It differs from number-to-text coercion (15 significant digits,
`docs/excel-semantics/numbers.md`, T-0107). Do not merge the two routines.

## Acceptance criteria
- [ ] The F line for `probes/number-text-exponent-digits.case.json Calc!D8` leaves
      eval/baseline/probes.txt. The 2 lines for A27 and A61 leave eval/baseline/poi.txt. No new F
      lines appear in any suite.
- [ ] The rule is recorded in `docs/excel-semantics/number-formats.md` ("`TEXT(x,"General")` uses an
      11-character fixed-notation limit"), citing these cells. The open D8 sentence in numbers.md
      ("Exponent width") is updated.
- [ ] Unit tests in the formatter cover D8, A27, A61 and the passing boundary cells listed above.
- [ ] `make check` green; reviewers approve.

## Out of scope
Number-to-text coercion (T-0107). Booleans in TEXT (T-0935). Width-dependent cell display
(`formatted()` with column widths, M10).
