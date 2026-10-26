---
id: T-2797
title: "Probe where IMSINH/IMCOSH switch between libm and the 1/EXP(-x) kernel (0.01 to 0.0105, 18.2 to 100)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's two switch points between libm-like digits and the `1/EXP(-x)` IMSINH/IMCOSH kernel are pinned and recorded, so `INVERSE_EXP_LOW` and `INVERSE_EXP_HIGH` cites evidence.

## Context
- Left by T-2629. Excel's IMSINH/IMCOSH take sinh x and cosh x as `(1/u ∓ u)/2` with `u = EXP(-x)` over the probed
  band 0.0105 <= |x| <= 18.2 (probes/t2629-imcosh-imsinh-y-sweep, probes/t2778-imsinh-sign-sweep), but libm-like digits
  at 0.01, 0.00012 and 100 (probes/t2535-imsinh-imcosh-kernel-sweep A55, A6, B6; eng-im-overflow-edges-2 A60). The
  engine uses the form over 0.01 < |x| <= 20 (`INVERSE_EXP_LOW` and `INVERSE_EXP_HIGH` in functions/engineering/complex/trig.rs); both ends
  are guesses (20 is where the worksheet TANH turns to ±1).
- Design: IMSINH of real x on two runs, about 200 values each in (0.0095, 0.011) and (18, 101), at both signs, chosen
  where the kernel and libm differ in the 15th digit (screen candidates with Python first; the numbers.md 15-digit
  midpoint rule decides which differ). Also the form below the lower cut (libm, a Taylor series, or expm1).
- Same probe, while at it: past |x| = 709 `hyperbolic_log_form` still takes libm `y.sin()`/`y.cos()`, where the product
  form now takes Excel's SIN/COS (`trig_sin_cos`, T-2629); add a few 709 < |x| < 710 cells with y where the two differ.
- engineering.md "IMSINH, IMCOSH and IMSECH take sinh and cosh as (1/u ∓ u)/2" holds the evidence so far.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes/cases/ filled by excel-graph, and engineering.md records both cuts (or the
      narrowest interval the probe allows) with cell citations.
- [ ] If a cut differs from `INVERSE_EXP_LOW` and `INVERSE_EXP_HIGH`, a follow-up task to move it (or the fix in this PR if it is one line).
- [ ] `make check` green; reviewers approve.

## Also (T-2796 review)
IMSIN and IMCOS share this band since T-2796 (`trig_sinh_cosh`): add IMSIN/IMCOS of mixed arguments x+yi with |y| near
both cuts (0.0105, 18.2-100) so their switch is probed too.

## Out of scope
- Changing IMSIN/IMCOS (T-2796).
