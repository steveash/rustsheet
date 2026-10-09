---
id: T-1202
title: "Out-of-domain ACOS/ASIN/ACOSH nested in another function give #NUM!, not NaN"
milestone: M5
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An out-of-domain argument to ACOS/ASIN/ACOSH and similar `single_number_fn` functions yields `#NUM!` even when nested, never a NaN number.

## Context
From T-1192's gen-m5 run: gen-m5/cd883013bea3b305 Calc!A23 `=DEGREES(SERIESSUM(ACOS(Data!C9),0,...))` with C9=1E+15: Excel `#NUM!`, engine -5672.28 (the NaN from `ACOS(1E+15)` is turned into 1 by `x.powf(0)` in SERIESSUM). `fn_acos` is `single_number_fn!(fn_acos, |f| Ok(f64::acos(f)))` (functions/math_and_trigonometry/mathematical.rs, macro in functions/macros.rs). Check other domain-limited functions built on the macro. Semantics: docs/excel-semantics/numbers.md "Huge arguments of trig functions".

## Acceptance criteria
- [ ] gen-m5/cd883013bea3b305 A23 passes.
- [ ] A unit/eval case for each domain-limited function nested in another function.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
