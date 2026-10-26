---
id: T-2131
title: "IRR's returned iterate to 15 significant digits (50 cells right to only 13-14 digits)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
campaign: visible
depends_on: []
---

## Goal
IRR returns a value within half a unit of the 15th significant digit of Excel's on the probe and
workbook cells. Excel returns an un-converged iterate, so this means stopping where Excel stops,
not converging further.

## Context
- probes/xirr-irr-empty-guess A11, A13, A15, A17, A19, A21 (T-2688): IRR from guess 0 (`IRR(v,0)`, `IRR(v,)`)
  on -70000, 12000, 15000, 18000, 21000, 26000. Excel 0.08663094803653149, ours 0.08663094803653171.
- `financial/financial_util.rs` `compute_irr` runs:
  1. a secant in v = 1/(1+r) from v0 and v0 ± 1e-3, stopping when |Δv| < 1e-7 and |NPV| < 1e-7;
  2. one retry from 0.1;
  3. a Newton fallback.
- docs/excel-semantics/financial.md, "IRR, RATE and XIRR stop early": Excel's solvers return an
  un-converged iterate. T-1533 fitted part of IRR's update and stop rule (decision 0065).
- Visibly-off cells on the 2026-10-20 run:
  - `probes-numeric/m9-fin-irr-rate`: 32;
  - `probes/solver-iteration-traces`: 14 (per-iterate traces);
  - `ironcalc-calc/FINANCIAL/IRR.xlsx`: 8, e.g. L17 `IRR(A17:F17)` Excel -0.6737366751808405,
    ours -0.6737366752054068;
  - `probes/fin-argument-modes`: 2.
- The 24 NPV cells of IRR.xlsx (`NPV(L2,A2:F2)` and similar) evaluate NPV at IRR's root. They
  are near-zero residues that follow the IRR value and should move with it.

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): IRR values of the financial sweep: most off in the 12th to 15th digit, and `IRR(Data!C1:E7)` of 31474c20fc1dafd6 A173 is 661.4076391219855 in Excel where ours finds another root (-0.9928).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 16 known-failure lines in eval/baseline/gen-m9.txt, 16 visibly off.
Examples:
- gen-m9/31474c20fc1dafd6 Calc!A173 `=IRR(Data!C1:E7)`: Excel 661.4076391219855, ours -0.9928069001823069
- gen-m9/31474c20fc1dafd6 Calc!A271 `=IRR(Data!D5:E8)`: Excel -0.7467044766537371, ours -0.7467044766454181
- gen-m9/67a4c03afd63f988 Calc!A346 `=IRR(Data!B4:D6)`: Excel 7026.083662734896, ours 7026.0836627358585
- gen-m9/9d9ceba257f8001b Calc!A120 `=IRR(Data!A3:D7)`: Excel 1999.0000006215012, ours 1999.0000006215882
- gen-m9/a91bad4e84f49739 Calc!A369 `=IRR(Data!C5:E5)`: Excel 6.909808543634389, ours 6.909808543634612
- gen-m9/b73c6cc96e093f03 Calc!A2 `=IRR(Data!B5:D11)`: Excel 1018.6181025376804, ours 1018.6181025376522
Cells:
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A285
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A164
- gen-m9/31474c20fc1dafd6.case.json (3): Calc!A173, A222, A271
- gen-m9/67a4c03afd63f988.case.json (4): Calc!A3, A199, A297, A346
- gen-m9/79d87fc527927626.case.json (1): Calc!A173
- gen-m9/9d9ceba257f8001b.case.json (1): Calc!A120
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A369
- gen-m9/b73c6cc96e093f03.case.json (1): Calc!A2
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A373
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A218
- gen-m9/d0854366830dcdee.case.json (1): Calc!A216

Last-digit cells of the same sweep (not visibly off; re-check them after the fix):
Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 12 known-failure lines in eval/baseline/gen-m9.txt, 12 last-digit (not visibly off).
Examples:
- gen-m9/1cd3096e3e87c965 Calc!A81 `=IRR(Data!A3:C6)`: Excel 5.132237590535713, ours 5.132237590535714
- gen-m9/5422bdbf173fc233 Calc!A14 `=IRR(Data!D4:E6)`: Excel 0.19784497071610319, ours 0.19784497071610296
- gen-m9/7d2979fba2c95d64 Calc!A20 `=IRR(Data!B1:C6)`: Excel 3.0625866844984957, ours 3.0625866844984966
Cells:
- gen-m9/0a957119475760c5.case.json (1): Calc!A48
- gen-m9/1cd3096e3e87c965.case.json (1): Calc!A81
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A124
- gen-m9/5422bdbf173fc233.case.json (1): Calc!A14
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A202
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A20
- gen-m9/81bcdc0bec0c8e6b.case.json (1): Calc!A268
- gen-m9/abb7d565af89b849.case.json (1): Calc!A107
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A53
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A154
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A169
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A3

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 16 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The 12 T-2257 last-digit lines in Context are re-checked after the fix; the Outcome says how many pass.
- [ ] Every IRR cell listed above is within half a unit of the 15th significant digit. The fitted
      rule (update, stop test, fallback path) is recorded in financial.md with its cells.
- [ ] The IRR.xlsx NPV residue cells are re-measured and the Outcome states where they land.
- [ ] Regressions follow decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR (T-2045).
- RATE (T-2005).
- IRR's error and lift edge cases (T-1686, T-2103).

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083). Bit-exact iterates are a follow-up
if the stage-1 fit does not reach them.
