---
id: T-3122
title: "Probe ODDFYIELD and ODDLYIELD at a price far from par where the solver nears the discount overflow"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
ODDFYIELD and ODDLYIELD match Excel for a price far from par, where the solver's trial yields make `(1+yld/frequency)^t` overflow, established by a probe.

## Context
T-2659 found ODDFPRICE is `#NUM!` once the discount power overflows and ODDLPRICE never overflows (financial.md "ODDFPRICE and ODDLPRICE at a huge yield", probes/odd-bond-huge-yield-overflow). ODDFYIELD's solver (`yield_solver.rs`, `odd_first_yield`) prices with `discount_growth` and so does not apply that rule on its trial yields; ODDLYIELD is closed-form. Neither was probed (T-2391 covered YIELD).

## Acceptance criteria
- [ ] A probe case in eval/suites/probes with excel-graph values brackets prices where ODDFYIELD and ODDLYIELD hit the boundary; the rule is in financial.md and every cell passes or is a recorded known failure with an owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/odd-yield-far-from-par (102 cells, excel-graph) runs ODDFYIELD and ODDLYIELD from price 1E-300 to 1E15 for
semiannual, annual, basis 1 and a 1E300 redemption. ODDLYIELD never errors; ODDFYIELD is `#NUM!` from a price of 1E5 (a
number at 3E4) and saturates at a vanishing price. The engine already matched every cell, so no code changed; the rule is in
financial.md ("ODDFYIELD and ODDLYIELD at a price far from par").
