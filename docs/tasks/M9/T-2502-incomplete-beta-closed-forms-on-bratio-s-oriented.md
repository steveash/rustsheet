---
id: T-2502
title: "Incomplete beta closed forms on bratio's oriented arguments, with F's argument forms (T-2464 E1)"
milestone: M9
lane: functions-stat
status: done
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
  exp and 1 - v. Also `fisher.rs` (the x and y forms) and `beta.rs`. Since T-2119 (2026-10-25) BETA.DIST's cdf is
  `incomplete_beta::beta_reg` too, so its closed-form cells (b0 = 1, or a0 = 1 with y0 <= 0.9) move with this task;
  the rest waits for T-2505.
- Overlap:
  - T-2221 (F last digits: closed forms, complement and tail choice) is partly this task. Note in T-2221 what this
    task settles when it lands.
  - T-2219 (F to 15 digits) is mostly T-2505 and T-2506.

## Acceptance criteria
- [x] With the engine, w5's closed-form cell set (`closed_final.py`) is 1,286 of 1,286 bit-exact, including the T df 2
      cells. Where T stays on `student_t.rs`, its df-2 closed form must agree with this one.
- [x] ~~F.DIST, F.DIST.RT and FDIST use w5's x and y forms.~~ Met on every closed-form cell, which the visible-parity
      stage below owns (all S1 cells pass). Using the forms on the other F cells, and moving the unoriented callers,
      moved to T-2505 (its Context and a new acceptance criterion) per the Visible-parity stage rule below.
- [x] The rule's cut is pinned by unit tests at x0 = 0.0998999751 (general kernel) and x0 = 0.09999999999999998
      (closed form).
- [x] The cells r2-impact lists for step S1 are no longer visibly off, and no baseline gains a line.
- [x] `make check` green; reviewers approve.

## Out of scope
- bratio's other branches (T-2505 onwards).
- Moving the BETA.DIST cdf as a whole off statrs (T-2505).

## Visible-parity stage
Decision 0083: this task is done when every closed-form cell it owns (r2-impact step S1: 83 visibly-off pre-T2464
cells) is within half a unit of the 15th significant digit of Excel's value. Bit-level criteria still unmet then move
to a follow-up task without the campaign key.

## Outcome
Implemented on branch `claude/hopeful-meitner-5oxf0g-t2502` (merged with main after T-2119):
- `incomplete_beta.rs` `closed_form_tails(x, y, a, b)`: bratio's swap at x > 0.5 (a shape of 1 means min(a,b) <= 1),
  then b0 = 1: w = `excel_exp(a0 * excel_ln(x0))`; a0 = 1 with y0 <= 0.9: w1 = `excel_exp(b0 * excel_ln(1 - x0))`; the
  other tail (0.5 - w) + 0.5. `beta_reg_complement` and `beta_reg_upper` take it first, so BETA.DIST's cdf and
  BETA.INV's refinement (T-2119) use it. a0 = 1 with y0 > 0.9 keeps the old tail (`unoriented_shape_one`).
- F.DIST, F.DIST.RT, FDIST (`fisher.rs` `f_tail`): w5's x = 1/(1 + d2/(d1·X)), y = 1/(1 + d1·X/d2) into the closed
  forms where they apply; elsewhere the pre-T-2502 tail (d1X/(d1X + d2) forms, unoriented shape-1 forms). With w5's
  forms on every F cell, about 100 bit-exact F cells moved by 1 ulp on today's Cephes kernel (e.g.
  `F.DIST(1.5,5,5,TRUE)` probes/stat-argument-modes H215, `F.DIST(0.1,2,100,TRUE)` t2220-f-dist-sweep-cdf A191), so
  the forms everywhere wait for bratio's branches (T-2505). That is why criterion 2 is only partly met.
- F.INV/F.INV.RT/FINV (crossing search), F.TEST and BETA.INV's huge-shape crossing keep the pre-T-2502 kernel
  (`beta_reg_complement_unoriented`): the oriented forms moved `F.INV(0.05,2,5)` (m9-f-inv A10), the ironcalc
  `F_DIST.xlsx` J3/L3 inverses and F.TEST gen-m9/68d8ffb69ff739d8 A143 by 1-2 ulp. The inverses wait for T-2464 E8.
- T at df 2 (`student_t.rs` `closed_form_tail`) calls the same `closed_form_tails`, so it now uses the y0 <= 0.9
  cut, Excel's EXP/LN and (0.5 - w) + 0.5. BINOM.DIST at k = n - 1 (upper tail of I_p(k+1, n-k)) and NEGBINOM.DIST
  at r = 1 or f = 0 (I_p(r, f+1)) take the closed forms; BINOM's k = 0 is left to T-2645.
- Verified: all 1,286 cells of w5's `closed_final.py` set are bit-exact through the worksheet functions (a temporary
  test over the dumped set; a 24-cell sample and the cut are unit tests in `incomplete_beta.rs`). r2-impact's 79
  step-S1 cells (`offcells_pre.tsv`, branch `closed*`) all pass.
- Eval against main b4c266d56: 154 cells fixed (probes-numeric 144, gen-m9 5, ironcalc-calc 5; 23 of them were
  visibly off), 0 newly failing. One cell still failing has a changed value: t1543-inverse-cdf-tiny-p A73
  `BETA.INV(1E-300,1,1)`, still visibly off and a little closer to Excel. `make eval-baseline` removed 10 lines
  (gen-m9 5, ironcalc-calc 5). Before the merge (statrs BETA.DIST cdf) the same change fixed 276 cells, but it broke
  the chained `BETADIST_BETAINV.xlsx` H30; T-2119's crossing refinement on main resolves that.
- T-2221 now notes what this task settles.
- Left to T-2505 (noted in its Context): the parts of criterion 2 not met here. The w5 F argument forms are used only
  where a closed form applies, and the unoriented callers (F.INV/F.INV.RT/FINV with T-2511, BETA.INV's huge-shape
  crossing, F.TEST) still need moving to the port. statistics.md records both in the T-2464 closed-forms entry's
  Ours line; T-2502's own engine-status section was folded into that entry and its "Implemented in:" line.
- Review follow-ups: T-2658 probes bounded BETA.DIST/BETADIST (A, B given) at a shape of 1 near the x0 = 0.5 swap and
  the y0 = 0.9 cut, since `beta.rs` now sends the bounded path through the oriented closed forms at t = (x - A)/(B - A),
  y = fl(1 - t) and w5's evidence covers only the unbounded forms. T-2505's acceptance criteria now include deleting
  the pre-T-2502 seams. Function-level tests pin the BINOM.DIST (k = n - 1) and NEGBINOM.DIST (r = 1, f = 0) dispatch
  (`test/statistical/test_fn_binom.rs`).

