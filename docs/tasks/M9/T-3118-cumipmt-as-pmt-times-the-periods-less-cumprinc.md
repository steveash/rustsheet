---
id: T-3118
title: "CUMIPMT as PMT times the periods less CUMPRINC (and type 1 from period 1 through period 2), once PMT has Excel's bits"
milestone: M9
lane: functions-financial
status: blocked
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
bit-exact cell. Cells it should move: probes/fin-cumipmt-gate-edges A2:A8, E1:E10, H2:H4; the new probe's A, D, F, H,
K, O, Q, S columns; gen-m9 e168be7f6d3f84c2 A379. T-3067 closed with these left to this task too: the type 1 range
from period 1 `CUMPRINC_CUMIPMT.xlsx` G2 (= fin-cumipmt-overflow-and-forms A7, fin-cumipmt-gate-edges H5, H6) and gen-m9
79d87fc527927626 A352 `CUMIPMT(1E-15,283,(1.005+3.14159+1E15)%,2,16,0)` (3 ulps of `pmt·m`, no pairing of 10 payment
forms with 5 principal forms fits; financial.md "T-3067 notes").

## Acceptance criteria
- [ ] CUMIPMT is `PMT·m - CUMPRINC` (with the period-1 rule) wherever that keeps every bit-exact cell, or the cells
      it cannot take are documented in financial.md with the cause (PMT or CUMPRINC bits).
- [ ] No regressions (`make eval-baseline`); `make check` green; reviewers approve.

## Out of scope
- CUMPRINC's own last bits (T-3067, T-1374) and PMT's (T-3014).

## Blocked because
Blocked because: the change needs a one-off `--allow-regressions` baseline update (decision 0170, proposed) that this
run's session was not permitted to run.

The branch builds CUMIPMT as `PMT·m - CUMPRINC` (with the period-1 rule) and removes the per-period interest forms.
Measured with `rustsheet-eval check --suite probes --suite gen-m9 --suite ironcalc-calc` on the branch: 53 known
failures fixed (probes 39, gen-m9 7, ironcalc-calc 7), CUMIPMT visibly-off 102 to 67, no CUMPRINC cell moved, and 7
REGRESSIONS (decision 0170 lists them; 5 visibly off). Each regression is our CUMPRINC 1 to 2 ulps off Excel's on that
range, not the rule (financial.md "The interest as `PMT·m - CUMPRINC`, applied"). No structural gate keeps all 7
without giving up most fixes. Engine tests updated (`the_interest_is_pmt_times_the_periods_less_cumprinc`); `make
check` was not run to completion because the baseline update could not be recorded.

To unblock: a reviewer accepts decision 0170 (or rejects it, and T-3192 lands first so the 7 cells stop regressing);
then a run on this branch runs the decision's `update-baseline --allow-regressions` command, confirms the baseline
diff adds exactly the 7 lines, runs `make check` and the review panel.
