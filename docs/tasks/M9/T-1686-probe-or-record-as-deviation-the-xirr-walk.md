---
id: T-1686
title: "Probe Excel's IRR/XIRR caps, the IRR fallback path and overflowing trial rates"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence replaces the engine choices T-0916 made where Excel was never probed. Those choices are
the solver caps (now `DEVIATION` entries, financial.md "The IRR and XIRR solver caps"), IRR's fallback,
and trial rates whose power overflows.

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
- [ ] Probe cases (excel-probe skill) cover these groups:
  - flows where IRR's secant does not stop, including `IRR({-400,1,1,1,1,1})`: what does Excel return,
    a value or `#NUM!`?
  - cap-reaching XIRR walks and bisections;
  - overflowing trial rates: `XIRR({-1,1E+20,1},{45000,45365,63615},0.1)` (engine `#NUM!`) and
    `IRR({-1,0.5,0.5,0.5,0.5,0.5,0.5,0.5},1E+60)`.
- [ ] The engine follows the evidence. Caps that stay engine-only keep their DEVIATION entry, with an
  `eval/suites/<suite>/deviations` line where a probe cell exists (decision 0036).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the fitted models.
