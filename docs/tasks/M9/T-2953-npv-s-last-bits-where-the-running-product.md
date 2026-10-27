---
id: T-2953
title: "NPV's last bits where the running product misses (gen-m9 9d9ceba257f8001b A85 at rate 1E-10)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
defer: phase-1b
depends_on: [T-2672]
---

## Goal
NPV is bit-exact with Excel on gen-m9 9d9ceba257f8001b Calc!A85, or the cell is explained and recorded, without
losing any cell the running-product rule already matches.

## Context
T-2672 moved `excel_npv` to Excel's running-product rule (financial.md "NPV's residue at IRR's root is not the double
sum"), bit-exact on 382/382 npv-term-order, 88/88 m9-fin-nper-npv and 252/255 gen-m9 NPV cells. The one cell the rule
itself misses: A85 `NPV(1E-10,Data!A8,{-100,50,60,0,0},{0,0,0})` (Data!A8 empty) is 9.999999981999995 in Excel and
9.999999982000002 under the rule (4 ulp, within 15 digits, so not visibly off). (-100 + 50*(1/b) + 60*(1/b)^2)/b and
(-100 + 50*b^-1 + 60*b^-2)/b with b = 1+1E-10 give Excel's bit; neither has been tested against the 470 lab cells.
It may be a tiny-rate path (1+r rounds to 1 + 1.000000082740371E-10) or a lucky match. Lab:
`uv run rustsheet-fitlab annuity-npv` (tools/src/rustsheet_tools/fitlab/kernels/annuity.py).

## Acceptance criteria
- [ ] A probe of NPV at tiny rates (1E-12 to 1E-6, with three to eight flows that do and do not cancel) has Excel
      values, and the lab scores the running product against the A85 candidates on it and on the 470 cells.
- [ ] If one formulation matches more cells than the running product, the engine uses it and A85's line leaves
      eval/baseline/gen-m9.txt; otherwise financial.md records the counts and the task closes as explained.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR's rate (T-2131), which decides abb7d565af89b849 A387.
