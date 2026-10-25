---
id: T-2546
title: "BESSELI, BESSELJ and BESSELK past the order cap: Excel's 0 instead of #NUM! (gen-m9, 5 cells)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: opus
depends_on: [T-2139]
campaign: semantics
---

## Goal
BESSELI, BESSELJ and BESSELK past our order cap give Excel's value where it is 0, still without unbounded work.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep (docs/excel-semantics/engineering.md "Engineering functions in the gen-m9 engineering sweep (T-2258)" "BESSEL"). Excel: `BESSELI(100,100001)`, `BESSELI(2.5,1E+6)`, `BESSELI(8,1E+6)`, `BESSELJ(0,1E+6)` and `BESSELK(25019,100001)` (`BESSELK(EDATE(25080,-2),100001)`) are 0; ours `#NUM!` from `MAX_ORDER` (crates/rustsheet-engine/src/functions/engineering/transcendental/bessel_excel.rs), the DEVIATION "BESSEL order cap" in engineering.md (decision 0014). Returning 0 past the cap costs O(1), so directive 5 does not need the `#NUM!`. But x near the order is not 0: `BESSELJ(1E+5,100000)` is 0.009636944011336267, so the DEVIATION shrinks only where Excel is shown to be 0. T-2139 probes BESSELJ at orders 100001 to 2^31-1; add `BESSELJ(2E+5,100001)`, `BESSELI(2E+5,100001)`, `BESSELK(2.5,100001)` and `BESSELY(1E+6,100001)` to it (or to a probe here) and decide from them. Update the DEVIATION entry to what remains.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 5 known-failure lines in eval/baseline/gen-m9.txt, 5 formulas, 5 visibly off.
Examples:
- gen-m9/2f0d384b8ed0d392 Calc!A280 `=BESSELK(EDATE(25080,-2),100001)`: Excel 0.0, ours #NUM!
- gen-m9/4fffa5addab1f8af Calc!A171 `=BESSELI(100,100001)`: Excel 0.0, ours #NUM!
- gen-m9/516488da78a6b08c Calc!A256 `=BESSELJ(0,1E+6)`: Excel 0.0, ours #NUM!
- gen-m9/7096c4cd615d43ac Calc!A68 `=BESSELI(8,1E+6)`: Excel 0.0, ours #NUM!
- gen-m9/b5aef3645cec1b95 Calc!A213 `=BESSELI(2.5,1E+6)`: Excel 0.0, ours #NUM!
Cells:
- gen-m9/2f0d384b8ed0d392.case.json (1): Calc!A280
- gen-m9/4fffa5addab1f8af.case.json (1): Calc!A171
- gen-m9/516488da78a6b08c.case.json (1): Calc!A256
- gen-m9/7096c4cd615d43ac.case.json (1): Calc!A68
- gen-m9/b5aef3645cec1b95.case.json (1): Calc!A213

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2258): these 5 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The DEVIATION entry states Excel's behavior and ours past the cap after the change; per-cell work stays bounded.
- [x] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
Past `MAX_ORDER` BESSELI, BESSELJ and BESSELK now decide in O(1) instead of always `#NUM!`: 0 where Kapteyn's bound (J) or
the series bound (I) is below e^-850 and the Miller recurrence provably stays finite (from `ln Gamma`), and every order 0
for K from x = 705.5 (K0 and K1 flushed); Excel's own cap 10,000,000 is matched. Probe `probes/t2546-bessel-past-cap`
(83 cells, excel-graph) found Excel's tiny-x overflow (`BESSELJ(1E-5,100001)` `#NUM!`, `(2E-5,100001)` 0), I0's overflow
at x = 709.79 and the K start flush; a unit test checks the rules against the uncapped recurrences. Recorded in
engineering.md "BESSEL order cap" (DEVIATION shrunk to J near or past x = n and Y at large x). Eval: the 5 gen-m9 lines
gone; 8 stale probe deviations removed and the 6 remaining probe cells pinned as deviations (decision 0116); no new
known failures.
