---
id: T-3652
title: "ODDFPRICE and ODDFYIELD: probe and match date-order and domain validation across rate, yld, frequency and basis, against ODDL*"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
ODDFPRICE and ODDFYIELD give `#NUM!` where Excel does for every date ordering and argument domain, and Excel's visible value elsewhere across broad grids of rate, yield or price, frequency and basis, with the differences from the ODDL* pair (which mostly match) explained.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md, branch `claude/sharp-mendel-13x2km`). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports ODDFPRICE and ODDFYIELD returning values where Excel gives `#NUM!`, and visibly
different numbers across broad combinations of rate, yield, frequency and basis; the ODDL* pair mostly matches.
Done: T-2483 (argument check order of all four; ODDLPRICE before the last coupon), T-1447, T-1457, T-1490 (dates
near 9999), T-2007 and T-3152 (ODDFYIELD's solver), T-2207 (^ kernel), T-2659 and T-3122 (overflow). Open: T-3031
(ODDFYIELD slope for a long odd first period at frequency 2 and 4), T-3043 (last ulps), T-3625 (100th-check stop).
Not swept: every ordering of issue, settlement, first_coupon and maturity (equal dates, settlement before issue,
settlement on the first coupon, first_coupon past maturity, maturity off the coupon schedule), short versus long
odd first periods at each frequency, every basis, a negative or zero rate, yield or price, and a redemption of 0.
Compare each validation rule with ODDLPRICE/ODDLYIELD's, which already match, and with financial.md. Code:
functions/financial/financial_bonds.rs (`fn_oddfprice`, `fn_oddfyield`, `odd_first_yield`).

## Acceptance criteria
- [ ] A probe case (excel-graph) covers the date orderings and domain edges above at every frequency and basis.
- [ ] Every probed cell has Excel's type, and every number is within half a unit of Excel's 15th digit, except cells handed to T-3031 or T-3043 with a reason.
- [ ] financial.md states ODDF* validation next to ODDL*'s.
- [ ] Every quirk found is written up in docs/excel-semantics/financial.md in the same PR, with its probe cells cited.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Last-ulp fits (T-3043).
- ODDLPRICE/ODDLYIELD changes unless a probe shows a shared bug.
