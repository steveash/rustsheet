---
id: T-3756
title: "DURATION, MDURATION and PRICE last digits on T-2733's two fresh sweeps (120 cells, Phase 1b)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
DURATION, MDURATION and PRICE give Excel's bits on the cells below from T-2733's two R4.2 fresh sweeps. A cell whose
bits cannot be derived is listed in the Outcome with Excel, ours, the 50-digit math, and the candidates tried.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 26e0bd00b): these failing Excel cells are named by no open task.
  T-2733 (done) drew both sweeps after freezing its coupon-sum order. Its Outcome reports every point inside Gate A's
  envelope. All but one are within half a unit of Excel's 15th digit. That one is Calc!A119
  `PRICE(41960,42376,0.000630103,22.5622,2825.86,1,3)`, Excel 77.11398607113979, ours ...973. It is decision 0188
  R1(c) Phase 1b evidence. So the whole group is last-digit work on a dense sweep inside Gate A (R5), hence
  `defer: phase-1b`.
- **One coupon left (the n1 sweep, 42: DURATION in column B, MDURATION in C).** Here the closed form is a single
  coupon, and the misses are 1 or 2 ulp. Example: Calc!B106 `DURATION(38696,38758,232.89,0.013693,2,4)`, Excel
  0.16666666666666663, ours 0.16666666666666669. Excel's value sits below 1/6 and ours above it, so the lead is the
  operation order of `DSR/E` and the division by f in the N = 1 branch of `duration_of`. Check MDURATION's division by
  `1 + yld/f` after that.
- **Several coupons (the r4 sweep, 78: PRICE A52, A119, A131, A133; DURATION in column B; MDURATION in C).**
  This is the same residue as T-2972 (43 cells of T-2733's bond case, Calc!B5 onwards): the order of the Macaulay sum and
  its division by the price. Examples: Calc!B172 `DURATION(37995,48661,7.84504,0.00137633,2,1)`, Excel
  14.665124025203177, ours ...18; Calc!A133 `PRICE(42277,42445,0.0125133,83.4819,7659.03,4,3)`, Excel
  25.681431772339423, ours ...447. If one change fixes both, do it with T-2972 and record the split in both files.
- Cells (`make eval-show CASE=<case>`):
  - probes-numeric/t2733-price-n1-fresh.case.json (42): Calc!B106:B107, B110, B114, B116, B118, B120, B124, B126,
    B131:B132, B136, B142, B146, B157:B158, B160, B163, B169:B170, B196, C106:C107, C110, C114, C116, C118, C120, C124,
    C126, C131:C132, C136, C142, C146, C157:C158, C160, C163, C169:C170, C196
  - probes-numeric/t2733-price-r4-fresh.case.json (78): Calc!A52, A119, A131, A133, B27, B57, B89, B104:B106, B109,
    B116, B123, B125, B141, B161, B167, B170, B172, B187:B188, B197, B200, B203, B206, B216, B226, B242:B243, B253,
    B257, B260, B267, B273:B274, B277:B278, B282:B284, B286, B293, B295, C27, C57, C89, C106, C109, C116, C125, C141,
    C161, C167, C170, C172, C187:C188, C197, C200, C203, C206, C216, C226, C242, C253, C257, C260, C267, C273:C274,
    C277:C278, C282:C284, C286, C293, C295
- Semantics: docs/excel-semantics/financial.md "PRICE, DURATION and MDURATION sum order at a huge coupon rate and
  yield". Scripts: docs/research/t2733/ (`gen_fresh.py`, `r4.py`). Code:
  crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (`duration_of`, `CouponSchedule::price`).
- Needs no oracle sweep to start. The two sweeps (500 bonds, 1500 points) are the fit set here. Any order chosen on
  them owes a third sweep drawn after the freeze.

## Acceptance criteria
- [ ] More of the 120 cells above are bit-exact and no listed cell gets further from Excel. probes-numeric is ungated
      (no baseline file), so the counts before and after come from the report. Every PRICE cell of
      T-2733's bond case (Calc!A1 onwards) stays bit-exact, and no baseline under eval/baseline/ gains a line. Otherwise,
      record the candidates scored in financial.md (time-boxed, decision 0056).
- [ ] decision 0188 R4 guards (reviewer-excel-fidelity check 7) for any order or constant chosen by scoring:
      - Hold-out counts (bit-exact, within 4 ulp, within 15 digits) for the chosen and the correctly rounded
        candidate, on a split and margin fixed before scoring.
      - A fresh probes-numeric sweep drawn after the freeze and oracle-filled on excel-graph.
      - Its Gate A envelope count against attached mpmath values, with every point outside listed.
      - A `FITTED(T-3756, n_cells, range)` tag and range on any fitted constant. A structural order needs no tag.
      - Nothing fitted below the reproducibility floor (4 ulp web against desktop).
      - No keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 43 cells of T-2972 and the 6 of T-2973, except as a shared fix recorded in both. YIELD (T-1592).

`defer: phase-1b`: every cell here is inside Gate A, and all but one are within Excel's 15 digits (decision 0188 R5).
