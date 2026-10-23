---
id: T-2502
title: "Incomplete beta closed forms on bratio's oriented arguments, with F's argument forms (T-2464 E1)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
The engine's incomplete beta takes Excel's closed forms on bratio's oriented arguments, and F.DIST, F.DIST.RT and FDIST
pass the kernel Excel's x and y. Every closed-form cell of BETA.DIST, F, T at df 2, BINOM and NEGBINOM becomes bit-exact.

## Context
- T-2464 research, step E1 of docs/research/t2464/r2-impact/RESULTS.md section 4.
- docs/excel-semantics/statistics.md "Incomplete beta closed forms act on the oriented arguments (T-2464)" and
  "Incomplete beta is TOMS 708 `bratio`" give the rules.
- First swap:
  - when min(a,b) <= 1, to (b, a, y, x) if x > 0.5;
  - when min(a,b) > 1, if λ = a - (a+b)x < 0.
- Then the closed forms, with Excel's EXP and LN:
  - b0 == 1: w = EXP(a0·LN x0);
  - a0 == 1 and y0 <= 0.9: w1 = EXP(b0·LN(fl(1 - x0))), with 1 - x0 recomputed;
  - the other tail is (0.5 - w) + 0.5.
- F passes x = 1/(1 + d2/(d1·X)) and y = 1/(1 + (d1·X)/d2) (w5: 197 of 197 b0 = 1 cells, against 180 for the
  engine's d1X/(d1X + d2)).
- Evidence: 1,286 of 1,286 cells (w5-triangulation `closed_final.py`, verify-w5-triangulation/verify_closed.out).
  r2-impact's positive control passes 1,312 of 1,312 through every wrapper.
- r2-impact's expected change: -83 visibly-off pre-T2464 cells (-39 on its hold-out) and -14 in the w2f probes, with 0
  broke and 0 bit-exact cells lost.
- Code: `functions/statistical/incomplete_beta.rs`. Today it applies b = 1 and a = 1 before any swap, with std ln and
  exp and 1 - v. Also `fisher.rs` (the x and y forms) and `beta.rs`. BETA.DIST's cdf is still statrs (T-2119), so this
  task sends BETA.DIST's closed-form cells (b0 = 1, or a0 = 1 with y0 <= 0.9) to the new closed forms and leaves the
  rest on statrs until T-2505.
- Overlap:
  - T-2221 (F last digits: closed forms, complement and tail choice) is partly this task. Note in T-2221 what this
    task settles when it lands.
  - T-2219 (F to 15 digits) is mostly T-2505 and T-2506.

## Acceptance criteria
- [ ] With the engine, w5's closed-form cell set (`closed_final.py`) is 1,286 of 1,286 bit-exact, including the T df 2
      cells. Where T stays on `student_t.rs`, its df-2 closed form must agree with this one.
- [ ] F.DIST, F.DIST.RT and FDIST use w5's x and y forms.
- [ ] The rule's cut is pinned by unit tests at x0 = 0.0998999751 (general kernel) and x0 = 0.09999999999999998
      (closed form).
- [ ] The cells r2-impact lists for step S1 are no longer visibly off, and no baseline gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- bratio's other branches (T-2505 onwards).
- Moving the BETA.DIST cdf as a whole off statrs (T-2505).

## Visible-parity stage
Decision 0083: this task is done when every closed-form cell it owns (r2-impact step S1: 83 visibly-off pre-T2464
cells) is within half a unit of the 15th significant digit of Excel's value. Bit-level criteria still unmet then move
to a follow-up task without the campaign key.
