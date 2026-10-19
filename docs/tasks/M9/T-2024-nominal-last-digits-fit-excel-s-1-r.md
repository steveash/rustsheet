---
id: T-2024
title: "NOMINAL last digits: fit Excel's (1+r)^(1/n) evaluation order"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2023]
---

## Goal
NOMINAL returns Excel's bits for the cached and swept cells, through a fitted formulation of
`n * ((1+r)^(1/n) - 1)`.

## Context
Gap triage 2026-10-19. Known failures in eval/baseline/ironcalc-calc.txt,
`ironcalc-calc/FINANCIAL/NOMINAL_EFFECT.xlsx` Sheet1: C8 `NOMINAL(A8,B8)` (Excel 0.4267276788012868, ours
...859), E4 (9.999999999999995 / ...998), E15 (5.999999999999998 / 6.0), E17 (7.9999999999999964 / 8.0).
E4, E15 and E17 read an EFFECT cell (D4, D15, D17); D4 and D17 change when T-2022 lands, so re-triage
after it. `fn_nominal` (crates/rustsheet-engine/src/functions/financial/rate_conversions.rs) uses
`((1+r).powf(1/n) - 1) * n`. Triage check over the 20 cached NOMINAL cells (plain doubles):
`powf` then `*n` 16/20, `n*` first 16/20, `(exp(ln(1+r)*(1/n)) - 1)*n` 17/20, `exp(ln(1+r)/n)` 13/20,
`expm1(log1p(r)/n)*n` 7/20. Excel's `^` with a non-integer exponent is `exp(y*ln x)` with extended
products (docs/excel-semantics/numbers.md "Non-integer exponents"), the first candidate to try through
`arithmetic::power`. Rule entry: docs/excel-semantics/financial.md "EFFECT and NOMINAL".
**Needs an oracle sweep:** fit against `probes-numeric/m9-nominal-sweep` (T-2023), never blind; time-boxed
(decision 0056): record every candidate's bit-exact and within-4-ulp counts.

## Acceptance criteria
- [ ] The NOMINAL lines above (those still failing after T-2022) leave eval/baseline/ironcalc-calc.txt,
      and the sweep's NOMINAL cells pass, or the Outcome lists the residue with the candidate table.
- [ ] No new F lines in any baseline.
- [ ] financial.md "EFFECT and NOMINAL" records the fitted order and the rejected candidates.
- [ ] `make check` green; reviewers approve.

## Out of scope
- EFFECT (T-2022).
