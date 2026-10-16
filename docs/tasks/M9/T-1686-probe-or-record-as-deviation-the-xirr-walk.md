---
id: T-1686
title: "Probe or record as DEVIATION the XIRR walk and bisection caps and the IRR step cap"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine's solver caps either match a probed Excel cap or are recorded as `DEVIATION` entries
(directive 5, decision 0052), not left as "DEVIATION candidate".

## Context
T-0916 ported the fitted XIRR and IRR models (financial.md "XIRR is a bracketing bisection", "IRR is a
secant in v"). The models have caps that Excel's own were never probed for. The engine therefore
bounds its loops itself:
- the XIRR bracket walk, 100 points per walk;
- the XIRR bisection, 200 steps;
- the IRR secant, 50 steps, with one retry from 0.1.

They are enough on every oracle cell. One more open path: where both IRR secant runs fail, the
engine falls back to the old Newton/bisection solver. Excel returns -0.6737366751808405 for
`IRR({-400,1,1,1,1,1})` (IRR.xlsx General!L17), and no step cap reproduces that (1,000 steps give
...431). Probe what Excel does there, too. financial.md calls the XIRR caps a DEVIATION candidate. The
T-0916 reviewers asked for the entries, since every directive-5 bound gets one (decision 0014).

## Acceptance criteria
- [ ] Either probe cases (excel-probe skill) that hit each cap give Excel's value and the engine matches
  it, or financial.md records each cap as `Status: DEVIATION` with the engine's pinned behaviour
  (decision 0052), plus an `eval/suites/<suite>/deviations` entry where a probe cell exists.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the fitted models.
