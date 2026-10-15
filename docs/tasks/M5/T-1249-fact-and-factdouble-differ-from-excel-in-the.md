---
id: T-1249
title: "FACT and FACTDOUBLE differ from Excel in the last digits for large n (FACT(170), FACTDOUBLE(297..300))"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
FACT and FACTDOUBLE return Excel's exact double for large n (bit-identical or within the documented tolerance).

## Context
Probes/fact-large-arguments (T-1233) A6 `FACT(170.9)` expects 7.257415615308004e306, engine gives 7.257415615307994e306; A9 `FACTDOUBLE(300)` expects 8.154414069380598e307, engine 8.154414069380588e307 ; also A1 `FACT(170)` (Excel ...004e306, engine ...994e306), A11 `FACTDOUBLE(297)` (expected 1.2552756225993057e304, engine ...067e304), A12 `FACTDOUBLE(298)` (expected 2.718138023126864e305, engine ...863e305). The unit test in test_fact_factdouble.rs checks formatted text only and does not cover exactness. Same family as MULTINOMIAL inexactness (T-1190): Excel's factorial is not the left-to-right product (probably a Gamma/table-based computation). Needs numeric fitting against the probe values; opus. Code: `fn_fact`, `fn_factdouble` in functions/math_and_trigonometry/mathematical.rs.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/c263b43217a5898b Calc!A14 `=DEGREES(FACT(MAX(Data!$E2)))` = DEGREES(FACT(42)): Excel 8.05009207373008e52, engine 8.050092073730078e52.
  The engine's FACT(42) shows as 1.4050061177528798e51, one ulp below the correctly rounded 42! (1.40500611775288e51); with the correctly
  rounded value, `x*180/pi` gives Excel's 8.05009207373008e52 but the engine's DEGREES formula `x*(180/PI)` gives ...079e52, so DEGREES
  (`fn_degrees`, mathematical.rs) may need `x*180/PI` too: check it against ironcalc-calc DEGREES_RADIANS.xlsx before changing it.

## Acceptance criteria
- [x] The new known failures of probes/fact-large-arguments leave eval/baseline/probes.txt.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Excel's FACT and FACTDOUBLE are one double product taken from n down. `fn_fact` and `fn_factdouble` now share
  `descending_product` (mathematical.rs), which is still `#NUM!` at the first overflow and bounded for a huge n.
- New probe `fact-product-order-sweep` (excel-graph, 420 cells): FACT 0..170, plus FACTDOUBLE for every n where ascending,
  descending and correctly rounded disagree. Descending matches 420/420, ascending 64, correctly rounded 76. Recorded
  in numbers.md "FACT and FACTDOUBLE of a huge argument".
- DEGREES needed no change: `DEGREES(FACT(42))` (gen-m7/c263b43217a5898b A14) passes with the new FACT(42).
- Eval delta: probes known failures 5579 → 5574 (fact-large-arguments A1, A6, A9, A11, A12), +1 case all passing;
  gen-m7 950 → 951 passing; regressions 0.
