---
id: T-3489
title: "PMT 1 to 3 ulp low in magnitude at rates between 2^-53 and 2^-52 (pmt-tiny-rate row 2, pmt-tiny-rate-near-2e-16 rows 2-6)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
PMT gives Excel's last bits at rates between 2^-53 and 2^-52 (about 1.11E-16 to 2.22E-16), where the engine's
discounted payment is 1 to 3 ulp smaller in magnitude than Excel's.

## Context
Found by T-2748 (financial.md "PMT is a discounted payment", tiny-rate paragraph). The discounted payment is bit-exact
on every cell of probes/pmt-tiny-rate (160 cells, rates 1E-16 to 1E-12, both types, five nper/pv/fv sets) except the
2E-16 row, and on probes/pmt-tiny-rate-near-2e-16 (30 cells, rates 1.1E-16 to 4E-16) misses exactly the rows inside
the band: 1.2E-16 to 2.1E-16. 1.1E-16 and 2.2E-16 to 4E-16 are bit-exact. Excel is larger in magnitude every time
(`PMT(2E-16,346,96301,42,0)` Excel -278.4479768786224, ours -278.44797687862234; `PMT(1.5E-16,1000,1000,0,0)` Excel
-1.0000000000000753, ours -1.000000000000075). In that band `1+rate` rounds up to 1+2^-52 while `2+rate` rounds
down to 2, so `growth_log`'s s = rate/(2+rate) is rate/2 exactly. What T-2748 ruled out: computing `2+rate` in the
x87 register before the division (294 regressions over probes, gen-m9 and ironcalc-calc CUMPRINC_CUMIPMT against 23
fixes, so Excel's s is the double quotient there). A plain-double model with `log1p(rate)` as L matches some of the band's cells but not all, so
the gap is not L alone.
- probes/pmt-tiny-rate.case.json (10): S!A2, B2, C2, D2, E2, F2, G2, H2, I2, J2
- probes/pmt-tiny-rate-near-2e-16.case.json (10): S!A2, A4, A6, B2, B3, B4, C2, C3, C4, C5

Code: `discounted_payment`, `discounted_balance`, `growth_log` (functions/financial/annuity.rs). The same L feeds IPMT,
PPMT and CUMIPMT, so a fix must keep their probe cells.

## Acceptance criteria
- [ ] The 20 cells above leave the baselines (`make eval-baseline`), with no new known failures.
- [ ] The band's form is in docs/excel-semantics/financial.md "PMT is a discounted payment".
- [ ] `make check` green; reviewers approve.

## Out of scope
- PMT's general last digits over the T-2242 sweeps (T-2730).
