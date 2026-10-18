---
id: T-1890
title: "CUMIPMT and CUMPRINC closed form: fit Excel's last digits over many periods"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1845]
---

## Goal
CUMIPMT and CUMPRINC return Excel's bits over many periods and near period 1E15, with one formulation for every
period count.

## Context
T-1845 made both functions O(1): up to 32 periods they keep main's per-period IPMT/PPMT sum (it matches
`ironcalc-calc/FINANCIAL/CUMPRINC_CUMIPMT.xlsx` G3, G9, G15, G26, which the closed form misses), past that, or when
`(1+r)^nper` overflows, a closed form in `log1p`/`expm1` (functions/financial/cumulative.rs;
docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over many periods"). The closed form matches 22 of the 30
cells of probes/fin-cumipmt-period-bound bit for bit. These miss:
- 1 to 6 ulps: A3 `CUMIPMT(1E-9,1E9,1,1,1E9,1)` (-0.5819767057476867, ours ...866), A5 `CUMIPMT(0.005,100000,1000,2,99999,1)`
  (-496512.41305908264, ours ...276), A7 `CUMIPMT(0.0001,5000,1000,1,5000,0)` (-270.7960117291623, ours
  -270.79601172916193), A8 `CUMPRINC(0.0001,5000,1000,1,5000,0)` (-1000, ours -1000.0000000000001), A12
  `CUMPRINC(0.004,1025,1000,1,1025,1)` (-1000, ours -999.9999999999999), A19 `CUMPRINC(0.01,1E15,100000,1E15-10,1E15,0)`
  (-10367.62824821948, ours ...473), A23 `CUMIPMT(0.001,2000,1000,1.5,1999.7,0)` (-1312.3961571024322, ours ...324).
- 48 to 64 ulps, near period 1E15: A18 `CUMIPMT(0.01,1E15,100000,1E15-10,1E15,1)` (-626.1106453272478, ours
  -626.1106453272532) and probes/fin-remaining-later-argument-ranges K3 `CUMIPMT(0.01,1E15,100000,1E15-10,1E15,0)`
  (-632.3717517805198, ours -632.3717517805271).
The short-range sum has its own misses (the CUMPRINC_CUMIPMT.xlsx cells in eval/baseline/ironcalc-calc.txt,
probes/fin-argument-modes H72 and H80, fin-remaining-later-argument-ranges K11), owned by T-1374. One formulation that
fits both the short and the long cells would replace the 32-period split. The fit search of T-1845 (pow vs
`exp(m·log1p r)`, the payment's and the balance difference's parenthesizations, `r` vs `expm1(log1p r)` as the
denominator) is described in financial.md; Excel's form is closer to `log1p`/`expm1` than to `(1+r)^n`.

## Acceptance criteria
- [ ] Probe the cells T-1845 kept or chose without evidence: a payment that overflows
  (`CUMIPMT(Z2,Z5,Z3,1,Z5,0)`, `CUMPRINC(Z2,10,Z3,1,10,1)` in test_fn_cumulative_lift.rs, `#NUM!` as on
  main), a per-period sum that overflows (`CUMIPMT(0.5,10,1E308,1,10,0)`, now the closed form), and
  a few periods past 2^53 at a tiny rate (`CUMIPMT(1E-15,2E16,1,1E16,1E16+8,0)`, now the closed form).
- [ ] The cells above pass (baseline lines removed), with no new failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The short-range misses (T-1374) unless the same formulation fixes them.
