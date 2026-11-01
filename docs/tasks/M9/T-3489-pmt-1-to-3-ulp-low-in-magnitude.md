---
id: T-3489
title: "PMT 1 to 4 ulp low in magnitude at rates 1.2E-16 to 2.1E-16 (pmt-tiny-rate row 2, pmt-tiny-rate-near-2e-16 rows 2-6)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
PMT gives Excel's last bits at sampled rates 1.2E-16 to 2.1E-16, where the engine's discounted payment is 1, 2 or 4
ulp smaller in magnitude than Excel's, once the gap is shown to be above the oracle's reproducibility floor.

## Context
Found by T-2748 (financial.md "PMT is a discounted payment", tiny-rate paragraph). The discounted payment is bit-exact
on every cell of probes/pmt-tiny-rate (160 cells, rates 1E-16 to 1E-12, both types, five nper/pv/fv sets) except the
2E-16 row. On probes/pmt-tiny-rate-near-2e-16 (30 cells, rates 1.1E-16 to 4E-16) it misses only on rows 1.2E-16 to
2.1E-16; 1.1E-16 and 2.2E-16 to 4E-16 are bit-exact. Excel is larger in magnitude every time
(`PMT(2E-16,346,96301,42,0)` Excel -278.4479768786224, ours -278.44797687862234; `PMT(1.2E-16,12,1000,0,0)` Excel
-83.33333333333343, ours -83.33333333333337, 4 ulp). The band is not the 2^-53 to 2^-52 interval where `1+rate` rounds
up while `2+rate` rounds down: 2.2E-16 is inside that interval and bit-exact. What T-2748 ruled out: computing `2+rate`
in the x87 register before the division in `growth_log` (294 regressions over probes, gen-m9 and ironcalc-calc
CUMPRINC_CUMIPMT against 23 fixes, so Excel's s is the double quotient there). A plain-double model with `log1p(rate)`
as L matches some of the band's cells but not all, so the gap is not L alone.

Reproducibility floor (decision 0188 R4.4): both sidecars are Excel for the web (excel-graph), and every gap is 1 to
4 ulp, inside the web-vs-desktop floor of up to 4 ulp (docs/research/oracle-calibration.md). Before fitting a form,
show the bits are stable: a desktop fill (M14) or a second excel-graph fill of the same cells that reproduces them.
If they do not reproduce, close this task as below the floor.
- probes/pmt-tiny-rate.case.json (10): S!A2, B2, C2, D2, E2, F2, G2, H2, I2, J2
- probes/pmt-tiny-rate-near-2e-16.case.json (10): S!A2, A4, A6, B2, B3, B4, C2, C3, C4, C5

Code: `discounted_payment`, `discounted_balance`, `growth_log` (functions/financial/annuity.rs). The same L feeds IPMT,
PPMT and CUMIPMT, so a fix must keep their probe cells.

## Acceptance criteria
- [ ] The bits are shown stable above the floor (see Context), or the task is closed as below it.
- [ ] The 20 cells above leave the baselines (`make eval-baseline`), with no new known failures.
- [ ] The band's form is in docs/excel-semantics/financial.md "PMT is a discounted payment".
- [ ] `make check` green; reviewers approve.

## Out of scope
- PMT's general last digits over the T-2242 sweeps (T-2730).
