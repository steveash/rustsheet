---
id: T-1261
title: "GCD and MULTINOMIAL: a text or boolean range cell is #VALUE! even after a negative argument (2 gen-m6 cells)"
milestone: M5
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GCD and MULTINOMIAL return `#VALUE!` when a range cell is text or a boolean, even when an earlier argument is negative, as Excel does (the engine stops with `#NUM!` at the negative value).

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09):
- gen-m6/5cc04d6f42b23b51 Calc!A11 `=(GCD(((-1)/Data!C$11),Data!B1:B11)-1E+15)`: the first argument is -1 and B1 holds the
  text "TRUE". Excel `#VALUE!`, engine `#NUM!`.
- gen-m6/2c1a6e93a1974f48 Calc!A35 `=(-MULTINOMIAL(Data!A6:D9)-(..))`: the range's first cell A6 is -2.5, later cells hold
  FALSE (A9) and "Hello World" (D8). Excel `#VALUE!`, engine `#NUM!`.
So the type check over every argument (coercion.md "GCD / LCM / MULTINOMIAL range cells": text and booleans in a range
are `#VALUE!`) wins over the negative-number `#NUM!`, whatever the order. Whether a typed text argument after a typed
negative (`GCD(-1,"a")`, engine `#NUM!`) and LCM behave the same is unprobed; add those cells with the excel-probe skill
if you change their path. Code: `fn_gcd`/`fn_lcm` (math_and_trigonometry/gcd_lcm.rs) and `fn_multinomial` (math_and_trigonometry/multinomial.rs) under crates/rustsheet-engine/src/functions/.
Record the precedence in docs/excel-semantics/coercion.md "GCD / LCM / MULTINOMIAL range cells" (the gen-m6 evidence was
added by T-1239's triage).

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/4629e931955aa4b8 Calc!A7 `=GCD(Data!A6:C11)` (B9 = -0.0001 comes before the text cells A10 "a*" and B10 "?b" and the boolean C11): Excel `#VALUE!`, engine `#NUM!`.

## Acceptance criteria
- [ ] gen-m6/5cc04d6f42b23b51 A11 and gen-m6/2c1a6e93a1974f48 A35 pass in `./target/release/rustsheet-eval show <case>`,
      with no new failing gen-m6 cell and no new known failures in eval/baseline/.
- [ ] Unit tests cover `GCD(-1, <range with a text cell>)` and `MULTINOMIAL(<range: -2.5 first, then a boolean>)`.
- [ ] gen-m7/4629e931955aa4b8 A7 passes. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] `make check` green; reviewers approve.

## Out of scope
- MULTINOMIAL's inexact values and numeric text (T-1190).
