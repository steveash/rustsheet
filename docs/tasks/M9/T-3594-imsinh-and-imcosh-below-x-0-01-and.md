---
id: T-3594
title: "IMSINH and IMCOSH below |x| 0.01 and IMSIN and IMCOS above |y| 20: Excel's sinh and cosh outside T-2778's band (5 t3525 fresh-sweep text misses)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3593]
campaign: visible
---

## Goal
IMSINH, IMCOSH, IMSIN and IMCOS give Excel's text outside T-2778's reciprocal-form band (|t| <= 0.01 and |t| > 20).
The kernel there is chosen from ranked candidates on T-3593's sweep, not on these 5 cells alone.

## Context
- Gap triage 2026-11-01 (`make eval` on main 408444f20, excel-graph). After T-3545, probes-numeric/
  t3525-im-hyperbolic-fresh-sweep passes 995 of its 1,000 cells. The other 5 are text cells, so decision 0188 R1(c)
  counts them as visibly off (not Phase 1b evidence), and no open task names them. `exit_owners.py --milestone M9`
  lists them under `rate:IMSINH`, `rate:IMCOSH`, `rate:IMSIN` and `rate:IMCOS`. They are 15th-digit misses of the
  sinh/cosh factor. Excel vs ours (`make eval-show CASE=probes-numeric/t3525-im-hyperbolic-fresh-sweep.case.json`):
  - S!A12 `IMSINH("-0.00878860151333914")`: Excel -0.00878871465166476, ours -0.00878871465166477 (libm sinh).
  - S!A7 `IMSINH("0.00313307442770563-0.855889226338187i")`: Excel 0.00205388173561626-0.755157850869937i, ours
    0.00205388173561625-0.755157850869937i (the real part sinh x·cos y).
  - S!B16 `IMCOSH("0.001258575663522")`: Excel 1.00000079200646, ours 1.00000079200645 (libm cosh).
  - S!C117 `IMSIN("0.981793650815856-183.167307387565i")`: Excel 1.47021409465799E+79-9.82267943792204E+78i, ours
    ...-9.82267943792205E+78i. S!D117 `IMCOS` of the same argument is the same factor (cosh y·cos x / sinh y·sin x).
- Two regions, each with its own selector in functions/engineering/complex/trig.rs:
  1. |x| <= `INVERSE_EXP_LOW` (0.01): `inverse_exp_sinh_cosh` takes libm `sinh`/`cosh` (A7, A12, B16). In the fresh
     sweep, 103 of the 106 IMSINH/IMCOSH cells with |x| < 0.01 match libm. engineering.md (T-3525) records the 3
     misses and says the cut 0.01 is not pinned by any cell. The reciprocal form cannot be the answer here:
     `IMSINH("0.01")` is libm's 0.0100001666675, and the form cancels to ...6674999.
  2. |y| > `INVERSE_EXP_HIGH` (20) in IMSIN/IMCOS: `trig_sinh_cosh` takes `exp_sinh_cosh`, the EXP pair with Excel's
     EXP (C117, D117; 92 of the 94 fresh-sweep cells there match). engineering.md (T-3545) lists C117 as "Still open".
- Candidates to rank, both regions: libm; the worksheet SINH/COSH kernel (`math_and_trigonometry::hyperbolic`, which
  is already Excel's for the scalar functions); the EXP pair; `EXP(t)/2` alone for |t| > 20 (EXP(-t) is below half
  an ulp there); and x87 extended-precision forms. T-3593's companion SINH/COSH/EXP cells give the full double at
  each argument. Comparing them with the 15-digit IM text shows directly whether IMSINH is SINH. Reuse the existing
  kernels (directive 3). Method: docs/research/excel-kernel-playbook.md. Fit lab: the `im-hyperbolic-fresh` and
  `im-small-imag` kernels (T-3525, T-3545).
- Needs an oracle sweep: T-3593. Do not pick a kernel from the 5 cells above.
- Cells owned:
  - probes-numeric/t3525-im-hyperbolic-fresh-sweep.case.json (5): S!A7, A12, B16, C117, D117

## Acceptance criteria
- [ ] The 5 owned cells match Excel's text, or each one left over is named in the Outcome with the candidate scores
      showing that no candidate within the guards below reaches it.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188), shown in the Outcome:
  - [ ] Hold-out (R4.1): train and hold-out counts side by side (bit-exact by text, within 15 digits) for the chosen
        kernel and the correctly rounded one, on T-3593's sweep split by the fit lab (T-3360), with the margin stated
        before scoring. The chosen kernel beats the correctly rounded one on hold-out.
  - [ ] Fresh sweep (R4.2): after the choice is frozen, a new probes-numeric case drawn log-uniform over both regions
        (or an `r4-T-3594` Gate A batch once T-3357/T-3358 land) is oracle-filled, and its counts are recorded.
  - [ ] Gate A envelope (R2): the fresh sweep's count inside the envelope (within 15 digits of Excel, or within 1e-9
        relative of 50-digit mpmath, absolute floor 1e-300). Every point outside is listed. The answer is 0 points
        outside and 0 error-against-number cells.
  - [ ] `FITTED` tag (R4.3): any constant or cut chosen by scoring (for example a new `INVERSE_EXP_LOW`) carries
        `FITTED(T-3594, n_cells, range)` and applies only inside that range. Outside it the code uses accurate math. A
        structural choice among candidates (kind (c)) has no tag, but the Outcome says it is kind (c).
  - [ ] Reproducibility floor (R4.4): no step targets differences below the oracle's floor (decision 0112, web against
        desktop up to 4 ulp).
  - [ ] No keep-matched branch: no band or special case whose domain is defined by the cells that already match
        (decisions 0056, 0082).
- [ ] Every other IMSINH/IMCOSH/IMSIN/IMCOS/IMSECH/IMCSCH/IMSEC/IMCOT oracle cell keeps passing: the fit lab's
      `im-hyperbolic` and `im-small-imag` sets, the t3545 sweeps and gen-m9. `make eval-baseline` records no
      regression.
- [ ] docs/excel-semantics/engineering.md's IMSINH/IMCOSH kernel entry and its T-3545 entry record the rule for both
      regions. They cite T-3593's case and drop "neither cut is pinned" / "Still open" where this settles them.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 and 0114 (`visible`): done when the 5 owned text cells equal Excel's text, or each leftover is recorded
as UNVERIFIED in engineering.md with the ranked candidates. Excel is the target, not the math. Bit-level residue that
the 15-digit text cannot show moves to a follow-up with `defer: phase-1b`.

## Out of scope
- |x| above 708 for IMSINH/IMCOSH (T-3546, in flight). The band 0.01 to 20 (settled by T-2778 and T-3545).
- IM* number-to-text notation (T-3560).
