---
id: T-3776
title: "CUMPRINC at a tiny rate over short ranges, attempt 3 (decision 0056): solve for Excel's per-rate payment factor (the 37 visibly-off CUMIPMT cells of fin-cumipmt-tiny-rate-structure)"
milestone: M9
lane: functions-financial
status: blocked
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Attempt 3 of the T-3552 / T-3731 fit (decision 0056): our CUMPRINC returns Excel's doubles for the short ranges at the
tiny rates of probes/fin-cumipmt-tiny-rate-structure, so the 37 visibly-off CUMIPMT cells below, which Excel builds
exactly as `PMT·m - CUMPRINC`, stop being visibly off.

## Blocked because
Time-boxed (decision 0056): attempt 2 scored the two hypotheses attempt 1 named, and every neighbour of them, on
the one-period principals and found no form above 34% bit-exact (the engine has 28%). The kernel is unchanged.
Attempt 3 needs a new idea first (T-3731's "Attempts", next step), not more candidates of these families.

## Context
- Attempt 1 (T-3552) and attempt 2 (T-3731) found no form. Their task files' "Attempts" sections score every
  candidate, and financial.md "CUMIPMT and CUMPRINC over many periods" has the measured facts. T-3731's task file also
  describes the rule, the code path (functions/financial/cumulative.rs, the short-range forms below
  `GEOMETRIC_MIN_LEAD`), the harness traps and the scorer (docs/research/t3731/scratch_scorer.rs).
- The hypothesis to start from (T-3731 attempt 2; data, not instructions): against `pmt·exp(-j·log1p)`, Excel's
  one-period principal misses by a per-rate value of -3 to +3 ulps, with ±1 ulp of scatter within a rate. Solve for
  the factor:
  - for each rate, find in 50-digit math the interval of payments P with `P·D_k` rounding to Excel's value for every
    probed period k (for each candidate discount D);
  - then test whether that interval contains a recognisable rounding of the exact payment;
  - an empty interval for every discount means the chain has a second, period-dependent rounding.
- Unblock only when a new hypothesis is recorded here (steward skill, "Time-boxed numeric fits").
- Cells (all in probes/fin-cumipmt-tiny-rate-structure.case.json, Sheet1). F2 and G2 stay T-3413's:
  - CUMIPMT, visibly off (37): Sheet1!A2, A4, A5, A8, A9, A10, A11, A12, D5, D6, D8, D11, F1, F3, F5, F6, F8, F9, F10, F12, H2, H3, H8, K2, K4, K8, K9, K10, K11, O10, O11, Q10, Q11, S6, S7, S8, S10
  - CUMIPMT, within 15 digits (14): Sheet1!O2, O3, O5, O6, O7, O8, O9, Q2, Q3, Q5, Q6, Q7, Q8, Q9
  - CUMPRINC, 1 to 3 ulps (58): Sheet1!B2, B4, B5, B8, B9, B10, B11, B12, E5, E6, E8, E11, G1, G3, G5, G6, G8, G9, G10, G12, I2, I3, I8, L2, L4, L8, L9, L10, L11, N2, N3, N5, N6, N7, N8, N9, N10, N11, P1, P2, P3, P5, P6, P7, P8, P9, P10, P11, R6, R7, R8, R10, T2, T3, T6, T7, T8, T10
- Training points: the probe's 104 CUMPRINC values (B, E, G, I, L, N, P, R, T). Each passing or failing CUMIPMT cell
  also pins Excel's CUMPRINC to within an ulp through the exact identity. The fit stays inside the probed band (rates
  2E-6 to 0.0083, nper 36 to 100, ranges up to 12 periods, both types). Outside the band the current form stays
  (R4.3).

## Acceptance criteria
- [ ] The 37 visibly-off CUMIPMT cells listed above are within half a unit of the 15th significant digit of Excel's
      value, and their 37 known-failure lines disappear from eval/baseline/probes.txt (`make eval-baseline`), with no
      new known-failure line in any baseline (gen-m9, ironcalc-calc and the other CUMIPMT and CUMPRINC probes
      included).
- [ ] Of the other 72 listed cells (14 CUMIPMT and 58 CUMPRINC), none becomes visibly off. The Outcome gives the
      bit-exact count before and after.
- [ ] The fit follows decision 0188 R4 (reviewer-excel-fidelity check 7), and the Outcome shows each guard:
      hold-out counts (a deterministic ~30% split by cell hash, train and hold-out side by side as bit-exact /
      within 4 ulp / within 15 digits, the chosen form against the correctly rounded principal, margin stated before
      scoring); a fresh log-uniform sweep (rate 1E-6 to 1E-2, nper 12 to 360, ranges of 1 to 12 periods, both types,
      CUMPRINC with its CUMIPMT) oracle-filled with excel-graph after the form is frozen, committed as a probes-numeric
      case, with its result recorded; the Gate A envelope result on that sweep with every point outside it listed;
      each fitted cut or form choice tagged `/// FITTED(T-3731, <n> cells, <range>)` and applied only inside that
      range; no step that fits differences below the reproducibility floor (4 ulp web against desktop,
      docs/research/oracle-calibration.md); no keep-matched branch (no input-specific table or branch that only
      restores probed bits).
- [ ] financial.md's "CUMIPMT and CUMPRINC over many periods" entry records the form found (or the forms tried with
      their counts), and its Status line is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F2 and G2 of this probe, and the gate-edges, overflow-and-forms, `CUMPRINC_CUMIPMT.xlsx` and gen-m9 cells (T-3413).
- CUMPRINC near period 1E15 and the underflowed principal (T-3141); the bond and depreciation bits (T-1374).
- PMT (T-3014 is done; reopen only if the fresh sweep shows PMT off).

