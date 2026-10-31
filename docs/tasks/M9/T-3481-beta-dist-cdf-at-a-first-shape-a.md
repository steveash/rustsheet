---
id: T-3481
title: "BETA.DIST cdf at a first shape a few ulps below 1: Excel is 1E-15 to 3E-15 off the math and moves with the shape's last bits (6 t2464-w2f-unit-shape cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3480]
campaign: visible
---

## Goal
BETA.DIST's cdf at a first shape a few ulps either side of 1 gives Excel's value to within half a unit of the 15th
significant digit (bit for bit where the fitted form allows), on the 6 cells below and on T-3480's sweep, by Excel's own
formulation there. Excel, not the math, is the target here: Excel is the one off the math.

## Context
Gap triage 2026-10-31 (`make eval` on main b9558688e). These cells were in T-2510's and then T-2512's scope (both done)
and no open task names them now:
- probes-numeric/t2464-w2f-unit-shape.case.json (6): Calc!A211, A407, A409, A411, A421, A423

Scored in triage against 50-digit mpmath: Excel is 9E-16 to 3.1E-15 relative off the math, and ours is 1.7E-16 to 9E-16.
For example, A409 `BETA.DIST(0.1,(67108863*2^-26+67108859*2^-52),5.0,TRUE)` (a = 1 - 5·2^-52) gives Excel
0.4095099999999998 and ours 0.4095100000000008. Excel's value moves with the shape's last bits (0.4095099999999996 at
A421, 0.40951000000000004 at a = 0.999999999999999). It is not the a = 1 closed form `1-(1-x)^b` (0.40950999999999993
in doubles), nor `-EXPM1(b*LN1P(-x))`. The cause is likely the formulation of bpser or apser for a0 just below 1
(statistics.md "Incomplete beta at a shape of 1 outside the closed forms", T-3007; the shape-of-1 w, T-3170).

Needs an oracle sweep: the 594 cached cells of the w2f case hold too few points at these shapes for a fit plus a
hold-out, so this task depends on T-3480. A fit is decision 0188 R4 work: kind (a) or (c) in reviewer-excel-fidelity
check 7.

## Acceptance criteria
- [ ] The 6 cells above are within half a unit of the 15th significant digit of Excel's value: after `make eval` none of
      them is among the visibly-off cells (`attributed_off`) of eval/runs/latest/report.json. No line is added to any
      eval/baseline/ file.
- [ ] Hold-out (R4.1): the fit is trained on T-3480's main case only. Train and hold-out counts (bit-exact, within 4
      ulp, within 15 digits) are shown side by side for the chosen form and for the correctly rounded value. The
      hold-out is T-3480's `-hold` case, or the fit lab's split once T-3360 lands. The margin is stated before any
      scoring. The chosen form beats the correctly rounded value on hold-out, and its hold-out exact rate is within the
      margin of its train rate.
- [ ] Fresh sweep (R4.2): after the form is frozen, a new log-uniform probes-numeric case over the whole near-unit-shape
      domain (or an `r4-T-3481` Gate A batch once T-3357 and T-3358 land) is oracle-filled, and its counts are in the
      Outcome.
- [ ] Gate A envelope (R2): on that fresh sweep, the count of points within 15 digits of Excel or within 1e-9 of
      50-digit math, with every point outside both listed (mpmath values attached until `rustsheet-eval accuracy`
      exists). No point leaves the envelope because of this change.
- [ ] `FITTED` tag (R4.3, for a kind (a) fit): each fitted piece carries `/// FITTED(T-3481, <n> cells, <range>)` and
      applies only inside that shape range, falling back to the accurate path outside it. quality/fitted.json changes
      only through `make hygiene`'s ratchet, never by hand.
- [ ] Reproducibility floor (R4.4): no step targets differences of 4 ulp or less, or bits that vary between fills.
- [ ] No keep-matched branch (decisions 0056, 0082): no band or special case whose only justification is that it
      keeps some cells matching.
- [ ] No probes-numeric or gated cell goes from within 15 digits to visibly off (per-case deltas in the Outcome).
- [ ] statistics.md records the form with the evidence counts; `make check` green; reviewers approve.

## Out of scope
- A shape of exactly 1 with w1 below 2^-53 (T-3150).
- Bit parity on cells already within 15 digits that the form does not settle (Phase 1b, decision 0188 R5).

## Visible-parity stage
Decisions 0083 and 0114 (`visible`, reproducing Excel's algorithm where Excel is the inaccurate one): done when the 6
owned cells are within half a unit of the 15th significant digit of Excel's value and the fresh sweep shows the
envelope. Time-boxed by decision 0056.
