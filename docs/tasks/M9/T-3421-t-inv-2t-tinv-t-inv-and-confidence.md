---
id: T-3421
title: "T.INV.2T, TINV, T.INV and CONFIDENCE.T at a tiny central probability, df 2 up: Excel's centre is 5 to 24 ulp off the exact quantile (CONFIDENCE.T A5 visibly off)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
Near the centre (two-tailed p near 1, T.INV's p near 0.5) at df 2 and up, the engine reproduces the digits Excel's
centre quantile gives, so `CONFIDENCE.T(0.999999999989522,1,3)` is within 15 significant digits of Excel's value.
Today the engine returns the exact quantile there and Excel does not.

## Context
- Gap triage 2026-10-31c (`make eval` on b1bdc1c2d, `make unowned-off`): probes/t3349-t-inv-2t-near-one A5 is a
  visibly-off cell with no open owner. T-3349 (done) found it and called it Excel's last digits. Its 33 other df >= 2
  failing cells are within 15 digits and also had no owner.
- Excel against the exact quantile (mpmath, 60 digits, the double p as given, triage scratch): Excel is 5 to 24 ulp off
  on 16 of the 34 cells (A1, A2, A4, A5, A13, A15, A19, A20, A23, A24, A25, A29, A34, A35, A42, A43) and 0 to 4
  ulp off on the other 18. The engine (`t_centre_quantile`, T-2724) is within 3 ulp
  everywhere. Excel's error grows as the central probability c = |1 - p| shrinks. Examples, Excel / ours / ulps of
  each from the exact value:
  - A1 `T.INV.2T(0.999999999989522,2)`: 1.481806159731988E-11 / 1.4818061597319863E-11 / +5, 0
  - A4 `T.INV.2T(0.999999999989522,30)`: 1.324203671942107E-11 / 1.3242036719421045E-11 / +14, -2
  - A5 `CONFIDENCE.T(0.999999999989522,1,3)`: 8.555211852081089E-12 / 8.555211852081079E-12 / +6, 0. This is A1's
    quantile divided by SQRT(3): Excel's A1 value over `SQRT(3)` gives Excel's A5 to the bit, and ours gives ours, so A5
    follows from the T.INV.2T digits alone.
  - A15 `T.INV.2T(0.999999999999999,2)`: 1.4130832128154019E-15 / 1.4130832128153975E-15 / +22, 0
  - A25 `T.INV.2T(0.999999999999999,30)`: 1.262789985640462E-15 / 1.262789985640457E-15 / +24, -1
  - A42 `T.INV(0.500000000001,2)`: 2.828364555072948E-12 / 2.8283645550729518E-12 / -11, -2
- docs/excel-semantics/statistics.md "Two-tailed Student-t quantile with p near 1 is accurate in Excel (T-3349)" holds
  to 15 digits. The ulp picture above is added to that entry in this triage. At df 1 Excel uses the 1/TAN form
  instead (T-2858 owns those cells).
- Code: `t_centre_quantile` and `T_CENTRE_C` in crates/rustsheet-engine/src/functions/statistical/t_dist.rs,
  `student_t_centre` in student_t.rs, `confidence_t_of` in normal.rs. T-3363 (todo, depends on T-3360) replaces
  the statrs inverse-beta port for these functions away from the centre. This task is the centre branch only, which
  that port never reaches. Whichever lands first re-triages the other.
- Evidence for the fit: the 34 cells below, plus the 76 points with |1 - p| below 1E-2 of T-3349's fresh sweep (the
  300-point probes-numeric case its Outcome names, excel-graph, seed 3349). Those points are training evidence only;
  this task does not own them.
- This is a structural model (decision 0188 R4 kind (c): which quantity Excel solves for, with what start and stop),
  or a fit if constants are needed. Differences of 4 ulp or less are under the reproducibility floor (R4.4) and are
  not targets. The 16 cells 5 ulp or more off are.

Cells (excel-graph, 2026-10-31):
- probes/t3349-t-inv-2t-near-one.case.json (34): Calc!A1, A2, A4, A5, A12, A13, A14, A15, A16, A17, A18, A19, A20, A22, A23, A24, A25, A26, A27, A28, A29, A30, A31, A32, A33, A34, A35, A37, A39, A40, A41, A42, A43, A44

## Acceptance criteria
- [ ] probes/t3349-t-inv-2t-near-one Calc!A5 is within half a unit of the 15th significant digit of Excel's value.
- [ ] Of the cells above that are 5 ulp or more off the exact value in Excel (listed in the Outcome), the bit-exact
      count goes up. The rest are named with their ulps. No cell above moves out of 15 digits of Excel.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7), each shown in the Outcome:
      - hold-out: a split stated before scoring (about 30% by cell hash, or the fit lab's once T-3360 lands), with train
        and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen model and for the
        correctly rounded quantile. The chosen model beats the correctly rounded one on hold-out, and its hold-out exact
        rate is within the stated margin of its train rate;
      - fresh sweep: after the model is frozen, a new probes-numeric case (log-uniform c = |1 - p| from 1E-15 to
        1E-2 on both sides of 1, df 2 to 1E6, and T.INV at 0.5 +- c) filled by excel-graph, with its counts recorded;
      - Gate A envelope on that sweep: every point within 15 digits of Excel or within 1E-9 relative of 50-digit
        mpmath (absolute floor 1E-300), with the count inside and every point outside listed, and no error against a
        number;
      - every fitted constant or cut-point carries `/// FITTED(T-3421, <n> cells, <range>)` and applies only inside that
        range, with the exact centre crossing outside it (`make hygiene` passes, quality/fitted.json updated by the
        ratchet's own command);
      - no step fits a difference of 4 ulp or less (the reproducibility floor, R4.4);
      - no keep-matched branch: no band or special case defined by, or tuned to, the cells that already matched.
- [ ] No gated cell regresses (`make eval-baseline` refuses regressions). The probes cells that become bit-exact
      leave eval/baseline/probes.txt, with no new known-failure line in any baseline.
- [ ] statistics.md's T-3349 entry states the centre rule found (or the candidates tried, with their counts).
- [ ] `make check` green; reviewers approve.

## Out of scope
- df 1 (T-2858), the tiny-p R3 question (T-3386), the statrs inverse away from the centre (T-3363).
- Bit parity on cells under the reproducibility floor: Phase 1b (decision 0188 R5).

## Visible-parity stage
Decision 0083 and 0114 (`visible`): done when Calc!A5 is within half a unit of the 15th significant digit of Excel's
value and the fresh sweep shows the Gate A envelope. Bit-level criteria still unmet then move to a follow-up with
`defer: phase-1b`. Time-boxed by decision 0056 (two attempts).
