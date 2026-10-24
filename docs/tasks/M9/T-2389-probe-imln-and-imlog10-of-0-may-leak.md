---
id: T-2389
title: "Probe IMLN and IMLOG10 of 0 (may leak -inf text)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
IMLN("0") and IMLOG10("0") answer what Excel answers (probably `#NUM!`), not a `-inf` text leak.

## Context
Found reviewing T-2334: imln/imlog10 in functions/engineering/complex.rs share the `r.ln()` path that leaked `-inf` for IMLOG2("0"). No oracle cell covers them; probe with the `excel-probe` skill first.

gen-m9 engineering sweep (gap triage 2026-10-23, T-2258; engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"): the probe is answered. `IMLN("0")`, `IMLN("0i")`, `IMLOG10("0")`, `IMLOG10("0i")` and both of a 0 or empty cell, or of a formula giving 0 (`FIXED(..)`, `INDEX(..)`, `SCAN(..)`, `IFS(..)`), are `#NUM!` in Excel; ours "-inf" (24 cells). So the fix can be made from these cells; a probe case is no longer needed.
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 24 known-failure lines in eval/baseline/gen-m9.txt, 18 formulas, 24 visibly off.
Examples:
- gen-m9/060b18586fcff6e6 Calc!A29 `=IMLN(Data!C9)`: Excel #NUM!, ours "-inf"
- gen-m9/2f0d384b8ed0d392 Calc!A271 `=IMLOG10("0i")`: Excel #NUM!, ours "-inf"
- gen-m9/40fc68fccfaf9b76 Calc!A205 `=IMLN(Data!Z1)`: Excel #NUM!, ours "-inf"
- gen-m9/493bb194c8e0b577 Calc!A108 `=IMLOG10(Data!Z1)`: Excel #NUM!, ours "-inf"
- gen-m9/493bb194c8e0b577 Calc!A311 `=IMLN(FIXED(((-0.0001)/Data!D1)))`: Excel #NUM!, ours "-inf"
- gen-m9/59a16b5ab566ee5c Calc!A96 `=IMLOG10(Data!C7)`: Excel #NUM!, ours "-inf"
Cells:
- gen-m9/060b18586fcff6e6.case.json (1): Calc!A29
- gen-m9/2f0d384b8ed0d392.case.json (1): Calc!A271
- gen-m9/40fc68fccfaf9b76.case.json (1): Calc!A205
- gen-m9/493bb194c8e0b577.case.json (2): Calc!A108, A311
- gen-m9/59a16b5ab566ee5c.case.json (1): Calc!A96
- gen-m9/62c886e21a1042af.case.json (1): Calc!A100
- gen-m9/76b07b57099ccc8d.case.json (3): Calc!A246, A252, A346
- gen-m9/7820ebc42c7a56a6.case.json (1): Calc!A136
- gen-m9/7c7e5bff12d93365.case.json (1): Calc!A33
- gen-m9/831f987e63e2412c.case.json (1): Calc!A308
- gen-m9/895a33aa8352864a.case.json (1): Calc!A373
- gen-m9/8c018ae063ac131b.case.json (2): Calc!A29, A170
- gen-m9/8fa8b572ebc1f001.case.json (1): Calc!A210
- gen-m9/a333e1febf9e93cd.case.json (1): Calc!A222
- gen-m9/a7fda34502ed3030.case.json (1): Calc!A237
- gen-m9/b5aef3645cec1b95.case.json (1): Calc!A8
- gen-m9/ca182d96dedee546.case.json (1): Calc!A353
- gen-m9/d9d55972b50e55e8.case.json (1): Calc!A207
- gen-m9/df083d85b436b49a.case.json (1): Calc!A170
- gen-m9/ee70a5515d004a66.case.json (1): Calc!A262

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 24 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Oracle values for IMLN("0") and IMLOG10("0") recorded; engine matches them, with a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
`IMLN` and `IMLOG10` of a zero argument now return `#NUM!` via `finite_complex_result` (as IMLOG2 does). Unit test `fn_imln_imlog10_of_zero_are_num`. Eval delta: 28 gen-m9 baseline known-failure lines removed (all the owned IMLN/IMLOG10 cells), regressions 0.
