---
id: T-3118
title: "CUMIPMT as PMT times the periods less CUMPRINC (and type 1 from period 1 through period 2), once PMT has Excel's bits"
milestone: M9
lane: functions-financial
status: done
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
- [x] CUMIPMT is `PMT·m - CUMPRINC` (with the period-1 rule) wherever that keeps every bit-exact cell, or the cells
      it cannot take are documented in financial.md with the cause (PMT or CUMPRINC bits).
- [x] No regressions (`make eval-baseline`); `make check` green; reviewers approve.

## Out of scope
- CUMPRINC's own last bits (T-3067, T-1374) and PMT's (T-3014).

## Outcome
- CUMIPMT is the engine's PMT times the period count less the engine's CUMPRINC (`Periods::interest`,
  functions/financial/cumulative.rs), a type 1 range from period 1 from the range from period 2; type 0's balance form
  and payments less principal, type 1's IPMT sum and tiny-rate form are gone (where PMT is `#NUM!`, type 0 keeps the
  payments less the remaining-balance principal). Engine test `the_interest_is_pmt_times_the_periods_less_cumprinc`.
- Eval delta (merged with main 2026-10-31, `make eval-baseline`): 53 known failures fixed (probes 39, gen-m9 7,
  ironcalc-calc 7), CUMIPMT visibly-off cells 102 to 67, no CUMPRINC cell moved; 7 cells regressed, exactly decision
  0190's list (5 visibly off), recorded once with `update-baseline --suite probes --suite gen-m9 --suite ironcalc-calc
  --allow-regressions` under decision 0190 (accepted 2026-10-31). Net +46 cells.
- The "No regressions" criterion holds under decision 0190, the ruling that allows these 7 (our
  CUMPRINC's last bits, amplified; financial.md "The interest as `PMT·m - CUMPRINC`, applied").
- Evidence kind: the identity is measured on Excel's own PMT and CUMPRINC outputs (T-3067 run 4, to the bit on
  60 pairs and 22 period-1 ranges), not chosen by scoring candidates against oracle cells; on the suites that played
  no part in deriving it (gate-edges, gen-m9, ironcalc-calc and the rest of probes) it fixes 53 cells and loses 7.
- Decision renumbered 0170 to 0190 and the follow-up T-3192 to T-3393, then to T-3413 (main took T-3393) on merging main (both ids were taken there).
- Follow-up: T-3413.
