---
id: T-2539
title: "IMLN, IMLOG10 and IMLOG2 at 1E+200 and 1E-200+1E-200i after T-2117, and IMPOWER of a base whose squares overflow (gen-m9, 10 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-2117]
campaign: visible
---

## Goal
IMLN, IMLOG10 and IMLOG2 of 1E+200 give numbers and of 1E-200+1E-200i and 1E+300+1E+300i `#NUM!`, as Excel does, and IMPOWER of a base whose squared modulus overflows is `#NUM!`.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)"). `IMLN("1E+200")` is 460.517018598809, `IMLOG10("1E+200")` "200" (3 cells), `IMLOG2("1E+200")` 664.385618977473 (ours "inf", "inf", `#NUM!`: x² overflows in `sqrt(x*x+y*y)`); `IMLOG10("1E-200+1E-200i")` (3) and `IMLOG10("1E+300+1E+300i")` are `#NUM!` (ours "-inf+0.34109408846046i", "inf+..."). T-2117 (done on origin/agent/functions-engineering/T-2117-imln-modulus, not merged) switches the three logs to `modulus` (hypot) with `#NUM!` when both parts are nonzero and x²+y² is infinite or subnormal: after it merges these 9 cells should pass; confirm. `IMPOWER("1E+300+1E+300i",-0.5)` is `#NUM!` (ours 7.76886987015019E-151-3.21797126452791E-151i, the power -0.5 is `SUBTOTAL(5,..)`), although the result is tiny: IMPOWER (hypot modulus, T-1950) also refuses a base whose x²+y² overflows. One cell: probe `IMPOWER("1E+300+1E+300i",-1)`, `("1E+300",-0.5)`, `("1E+300+1E+300i",0.5)` and `("1E+154+1E+154i",-0.5)` (excel-probe) before choosing the rule.
Code: `imln`, `imlog10`, `imlog2`, `fn_impower` in complex.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 10 known-failure lines in eval/baseline/gen-m9.txt, 6 formulas, 10 visibly off.
Examples:
- gen-m9/09e6a8028f5acfae Calc!A274 `=IMLOG10("1E+200")`: Excel "200", ours "inf"
- gen-m9/1068c44afcd56d53 Calc!A136 `=IMPOWER("1E+300+1E+300i",SUBTOTAL(5,Data!B11:E12,Data!D5:E12,Data!A8:B8))`: Excel #NUM!, ours "7.76886987015019E-151-3.21797126452791E-151i"
- gen-m9/12e4aa13a20cc1f8 Calc!A264 `=IMLN("1E+200")`: Excel "460.517018598809", ours "inf"
- gen-m9/40fc68fccfaf9b76 Calc!A212 `=IMLOG10("1E+300+1E+300i")`: Excel #NUM!, ours "inf+0.34109408846046i"
- gen-m9/76b07b57099ccc8d Calc!A64 `=IMLOG10("1E-200+1E-200i")`: Excel #NUM!, ours "-inf+0.34109408846046i"
- gen-m9/e85daedce5c2675e Calc!A87 `=IMLOG2("1E+200")`: Excel "664.385618977473", ours #NUM!
Cells:
- gen-m9/09e6a8028f5acfae.case.json (1): Calc!A274
- gen-m9/1068c44afcd56d53.case.json (1): Calc!A136
- gen-m9/12e4aa13a20cc1f8.case.json (1): Calc!A264
- gen-m9/40fc68fccfaf9b76.case.json (1): Calc!A212
- gen-m9/76b07b57099ccc8d.case.json (1): Calc!A64
- gen-m9/895a33aa8352864a.case.json (2): Calc!A132, A226
- gen-m9/9fa170a9f17624fd.case.json (1): Calc!A29
- gen-m9/e85daedce5c2675e.case.json (1): Calc!A87
- gen-m9/e882060cf4613302.case.json (1): Calc!A319

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 10 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The IMPOWER probe case is oracle-filled and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Note (T-2389)
T-2389 routed IMLN/IMLOG10 through `finite_complex_result`, which already fixed 4 of this task's cells (the IMLOG10 `1E-200+1E-200i` and `1E+300+1E+300i` ones: 40fc68fccfaf9b76 A212, 76b07b57099ccc8d A64, 895a33aa8352864a A226, 9fa170a9f17624fd A29). Re-triage the rest.

## Outcome
Re-triage: after T-2117 and T-2389 the IMLN/IMLOG10/IMLOG2 cells already passed; only gen-m9/1068c44afcd56d53 Calc!A136 (IMPOWER) was left. Probe probes/eng-impower-overflow (excel-graph): IMPOWER is `#NUM!` for a base with both parts nonzero whose x²+y² overflows; a pure real base is not refused. Implemented in `fn_impower`; recorded in engineering.md. Eval delta: gen-m9 known failures -1 (A136); new probe case passes.
