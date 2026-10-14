---
id: T-1202
title: "Out-of-domain ACOS/ASIN/ACOSH nested in another function give #NUM!, not NaN"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An out-of-domain argument to ACOS/ASIN/ACOSH and similar `single_number_fn` functions yields `#NUM!` even when nested, never a NaN number.

## Context
From T-1192's gen-m5 run: gen-m5/cd883013bea3b305 Calc!A23 `=DEGREES(SERIESSUM(ACOS(Data!C9),0,...))` with C9=1E+15: Excel `#NUM!`, engine -5672.28 (the NaN from `ACOS(1E+15)` is turned into 1 by `x.powf(0)` in SERIESSUM). `fn_acos` is `single_number_fn!(fn_acos, |f| Ok(f64::acos(f)))` (functions/math_and_trigonometry/mathematical.rs, macro in functions/macros.rs). Check other domain-limited functions built on the macro. Semantics: docs/excel-semantics/numbers.md "Huge arguments of trig functions".

More cells (T-1239's gen-m6 suite, excel-graph, 2026-10-09): gen-m6/3ee7ffc8d6ab061f Calc!A26
`=((..)&ADDRESS(ACOSH((-1)),4,SEARCH(Data!D$2," x ",5)))` is `#NUM!` in Excel; the engine gives `#VALUE!` because
`ACOSH(-1)` reaches ADDRESS as NaN, which `as i32` turns into row 0 (`ADDRESS(ACOSH(-1),1)` is `#VALUE!`,
`ADDRESS(SQRT(-1),1)` is `#NUM!`).

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/2bb922eef72e785e Calc!A30 `=CHOOSECOLS(((0/100)/(2.5+(-0.5))),ASIN(Data!D$9),Data!E4)` with D9 = 100: Excel `#NUM!`, engine `#VALUE!` (`CHOOSECOLS(0,ASIN(100))` is `#VALUE!`, `CHOOSECOLS(0,#NUM!)` is `#NUM!`: the NaN leaks).
- gen-m7/93976dd9a3578ed9 Calc!A13 `=(Data!A10<ASIN(PI()))`: Excel `#NUM!`, engine FALSE.

## Acceptance criteria
- [ ] gen-m5/cd883013bea3b305 A23 passes.
- [ ] A unit/eval case for each domain-limited function nested in another function.
- [ ] gen-m6/3ee7ffc8d6ab061f A26 passes.
- [ ] gen-m7/2bb922eef72e785e A30 and gen-m7/93976dd9a3578ed9 A13 pass. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] `make check` green; reviewers approve.

## Out of scope
-
