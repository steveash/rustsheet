---
id: T-0938
title: "TEXT General: 11-character fixed-versus-scientific choice for small numbers (2 known failures)"
milestone: M10
lane: functions
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(x, "General")` picks fixed or scientific notation for small numbers the way Excel does.

## Context
Evidence: `poi/GeneralFormatTests.xlsx` (excel-graph, T-0019), formula `=TEXT(Cn, Bn)` with
`Bn` = "General":
- A27 `1E-09` expected "0.000000001", actual "1E-09"; A61 `-1E-09` expected "-0.000000001", actual "-1E-09".
- Passing cells that bound the rule: A28 `1E-10` = "1E-10", A26 `0.01` = "0.01", A13 `1.2345678912` =
  "1.234567891", A17 `1E10` = "10000000000", A18 `1E11` = "1E+11".
Reading: General keeps fixed notation when the digits fit in 11 characters, and the minus sign
does not count (A61). This is the General *format*, which differs from number-to-text coercion
(15 significant digits, `docs/excel-semantics/numbers.md`, T-0107). Do not merge the two
routines. Formatter General: `crates/rustsheet-engine/src/formatter/`.

## Acceptance criteria
- [ ] The 2 lines (A27, A61) disappear from eval/baseline/poi.txt with no new F lines.
- [ ] Rule recorded in `docs/excel-semantics/number-formats.md` citing these cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
Number-to-text coercion (T-0107). Booleans in TEXT (T-0935).
