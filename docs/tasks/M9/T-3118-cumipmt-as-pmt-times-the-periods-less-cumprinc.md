---
id: T-3118
title: "CUMIPMT as PMT times the periods less CUMPRINC (and type 1 from period 1 through period 2), once PMT has Excel's bits"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3014]
campaign: visible
---

## Goal
CUMIPMT is built the way Excel builds it, from the engine's PMT and CUMPRINC, so its tiny-rate interests are
visibly right wherever PMT and CUMPRINC have Excel's bits.

## Context
T-3067 run 4 measured (probes/fin-cumipmt-tiny-rate-structure, docs/excel-semantics/financial.md "CUMIPMT and
CUMPRINC over many periods", T-3067 notes): `CUMIPMT = PMT·(e-s+1) - CUMPRINC` exactly in doubles, and a type 1 range
from period 1 is `CUMPRINC = PMT + CUMPRINC(2..e)`, `CUMIPMT = PMT·(e-1) - CUMPRINC(2..e)`. Taking that rule with
today's PMT takes 29 bit-exact cells off, because the engine's PMT rounds the exact payment once while Excel's is up to
0.9 ulp from it (M1, M2, M4, M5 of the probe; T-3014 fits Excel's PMT). Once T-3014 lands, re-run the experiment:
in `Model::cumulative_of` (functions/financial/cumulative.rs), CUMIPMT = `compute_payment(rate, nper, pv, 0, type)·m`
less the CUMPRINC kernel's value (for type 1 from period 1, the period-2 range), replacing `short_interest`,
`tiny_rate_interest`, `summed_interest` and `payments_less_principal`'s own payment forms where it keeps every
bit-exact cell. Cells it should move: probes/fin-cumipmt-gate-edges A2-A8, E1-E10, H2-H4; the new probe's A, D, F, H,
K, O, Q, S columns; gen-m9 e168be7f6d3f84c2 A379. T-3067 closed with these left to this task too: the type 1 range
from period 1 `CUMPRINC_CUMIPMT.xlsx` G2 (= fin-cumipmt-overflow-and-forms A7, gate-edges H5, H6) and gen-m9
79d87fc527927626 A352 `CUMIPMT(1E-15,283,(1.005+3.14159+1E15)%,2,16,0)` (3 ulps of `pmt·m`, no pairing of 10 payment
forms with 5 principal forms fits; financial.md "T-3067 notes").

## Acceptance criteria
- [ ] CUMIPMT is `PMT·m - CUMPRINC` (with the period-1 rule) wherever that keeps every bit-exact cell, or the cells
      it cannot take are documented in financial.md with the cause (PMT or CUMPRINC bits).
- [ ] No regressions (`make eval-baseline`); `make check` green; reviewers approve.

## Out of scope
- CUMPRINC's own last bits (T-3067, T-1374) and PMT's (T-3014).
