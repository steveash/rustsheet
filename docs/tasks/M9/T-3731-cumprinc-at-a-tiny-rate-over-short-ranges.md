---
id: T-3731
title: "CUMPRINC at a tiny rate over short ranges, attempt 2 (decision 0056): the 37 visibly-off CUMIPMT cells of fin-cumipmt-tiny-rate-structure, on T-3552's dyadic and rate-scan evidence"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Attempt 2 of T-3552's fit (decision 0056). On probes/fin-cumipmt-tiny-rate-structure, our CUMPRINC returns Excel's doubles for the short ranges at rates 2E-6,
2E-5, 1E-4 and 0.0083, so the 37 CUMIPMT cells listed below, which Excel builds exactly as
`PMT·m - CUMPRINC`, stop being visibly off.

## Context
- T-3552 attempt 1 (2026-11-01) found no form and changed nothing in the kernel. Its task file's "Attempts" section
  scores every candidate family. financial.md "CUMIPMT and CUMPRINC over many periods" (T-3552 attempt 1 bullet) has
  the measured facts:
  - Over one period, Excel's CUMPRINC is PPMT·(1+δL), where δL is the relative error of `growth_log`'s L. The slope
    is -1.04 over 60 rates (probes/fin-cumprinc-single-period-rate-scan, 720 cells).
  - The gap vanishes at power-of-two rates and stays at 3·2^-17 (probes/fin-cumprinc-dyadic-rates, 296 cells).
  - Against the correctly rounded payment `pmt·v^(n-k+1)` the bias is gone, but the spread is ±3 ulps.
  - No constant times any of 15 per-period discounts fits a single rate. Neither does a constant less Excel's IPMT.
    So Excel's one-period chain has at least two roundings that depend on the period.
- Start from attempt 1's hypotheses (data, not instructions; check them against this task):
  - Next hypothesis for attempt 2: since no constant times any per-period discount fits one rate, Excel's
    one-period chain has at least two roundings that depend on the period. Two candidates:
    - The interest built from a balance that carries no L error (r·B with B from `(1+r)^k` in double-double),
      subtracted from the exact payment.
    - The geometric sum evaluated on the period-1 and period-k discounts separately.
    Fit them first on the 60-rate scan (constant-C test per rate), then on the dyadic case.
- If this attempt does not raise the bit-exact count either, block the task "time-boxed (decision 0056)" with the
  candidates listed.
- Read oracle JSON with a round-trip parser (excel-kernel-playbook "Traps": serde_json without `float_roundtrip`
  misreads doubles). Use our PPMT, bit-exact on all 480 PPMT cells of the new cases, as the harness's positive
  control.
- Gap triage 2026-10-31k (`make eval` on origin/main, excel-graph evidence; `scripts/exit_owners.py --all-off` and a
  per-cell run of its `owned` rule over every failing cell, using the task files of origin/main and of the 54 task
  files on unmerged task-filing branches). These 37 cells are the M9 probe row's `rate:CUMIPMT` exit cells that no open
  task owns (`make exit-owners MILESTONE=M9`). T-3413 has the same root cause but lists only this case's F2 (with its
  CUMPRINC partner G2) and the gate-edges, overflow-and-forms, `CUMPRINC_CUMIPMT.xlsx` and gen-m9 cells. This task
  takes the rest of the paired probe. T-3413 and this task change the same CUMPRINC form, so the one that lands
  second starts by re-measuring its cells on the first one's main.
- Rule, measured (financial.md "CUMIPMT and CUMPRINC over many periods", T-3067 run 4; applied by T-3118, decision
  0197): Excel's `CUMIPMT(r,n,pv,s,e,t)` is `PMT(r,n,pv,0,t)·(e-s+1) - CUMPRINC(r,n,pv,s,e,t)` in doubles, to the bit
  on all 60 probed pairs. A type 1 range from period 1 is `PMT·(e-1) - CUMPRINC(..,2,e,1)`. The engine already
  computes it that way, and our PMT has Excel's bits on this probe (M1-M5 pass, T-3014). So every CUMIPMT cell
  here is off only because our CUMPRINC is 1 to 3 ulps from Excel's. An ulp of a principal of about 1,000 is 512 to
  4,096 ulps of an interest of 0.2 to 100.
- Excel's CUMPRINC is not its PPMT. Over a single period the probe's own pairs differ by 1 to 3 ulps: B1
  `CUMPRINC(2E-5,100,100000,3,3,1)` -999.0303135033106 against C1 `PPMT(2E-5,3,100,100000,0,1)` -999.0303135033108,
  B4 ...9652 against C4 ...9655, I1 -1662.087365975419 against J1 -1662.0873659754193. Our PPMT matches Excel on every C
  and J cell, so "CUMPRINC = PPMT for one period" is wrong. The single-period principal comes from a different
  form, which this task finds.
- Why `visible` and not `accuracy`: against 50-digit math (mpmath, triage scratch), Excel's interests are not
  correctly rounded, because the cancellation loses digits. A10 `CUMIPMT(2E-5,100,100000,12,12,1)` is 4.2E-14
  relative off in Excel and 1.1E-13 in ours. F1 `CUMIPMT(2E-6,100,100000,3,3,1)` is -5.7E-13 in Excel and +5.9E-13 in
  ours. K10 `CUMIPMT(1E-4,60,100000,3,12,0)` is 7.0E-14 in Excel and 2.9E-14 in ours. S8 `CUMIPMT(2E-5,100,100000,1,9,1)`
  is 2.1E-13 in Excel and 3.2E-14 in ours. The target is Excel's algorithm, and every cell is inside Gate A's
  envelope. This is a probe, not a dense sweep, and its cells are visibly off, so decision 0188 R5's Phase 1b
  deferral does not apply (T-3413 is not deferred either).
- Code: crates/rustsheet-engine/src/functions/financial/cumulative.rs. Every probed range has a leading exponent
  `(nper-end+1)·L` far below `GEOMETRIC_MIN_LEAD` (0.3): about 0.002 at 2E-5 and nper 100. So these principals take
  the short-range form below the geometric-sum cut, and that is the form to find. financial.md records what has
  been tried: with Excel's PMT bits, the best type 1 form from period 2 (the geometric sum on `log1p` or `growth_log`
  with Excel's expm1) is exact on 22 of 58 type 1 principals and within 3 ulps on the rest. CUMPRINC derived from the
  old interest forms regressed 10 to 42 CUMPRINC cells.
- Examples (Excel vs ours):
  - A10 `CUMIPMT(2E-5,100,100000,12,12,1)` -1.7801601439258548 vs -1.7801601439259684; its principal B10
    -999.2101533464493 vs -999.2101533464491.
  - H2 `CUMIPMT(1E-4,60,100000,4,4,0)` -9.50142364618091 vs -9.501423646181138; I2 -1662.2535747120169 (ours 1 ulp off).
  - O10 `CUMIPMT(0.0083,36,8000,1,11,1)` -569.5984192430335 vs -569.5984192430328; its range from period 2, N10
    `CUMPRINC(0.0083,36,8000,2,11,1)`, -1989.03775517299 (ours 1 ulp off).
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

## Visible-parity stage
Decision 0083 (`campaign: visible`, decision 0114): done when the 37 visibly-off CUMIPMT cells above are within half a
unit of the 15th significant digit of Excel's value. CUMPRINC cells still 1 to 3 ulps off after that move to a
follow-up filed with `defer: phase-1b` and named in the Outcome.
