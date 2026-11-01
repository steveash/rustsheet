---
id: T-3589
title: "IMSINH, IMCOSH, IMSIN and IMCOS: the 5 t3525 fresh-sweep texts one 15-digit tie off (|x| < 0.01 in the hyperbolic pair, |y| > 20 in the trig pair)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The 5 cells of probes-numeric/t3525-im-hyperbolic-fresh-sweep that T-3525 and T-3545 left failing show Excel's text.
If no kernel choice reproduces them without losing other cells, engineering.md records them as UNVERIFIED, with the
forms tried.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 408444f20; `scripts/exit_owners.py --all-off` lists them with no
  open owner). The sweep is 995/1000 after T-3545. Each miss is a last digit of a 15-digit part text, where the
  part's double sits near a rounding tie. Values below are mpmath at 40 digits on the double inputs:
  - **IMSINH/IMCOSH with |x| < 0.01** (libm's sinh/cosh, below `INVERSE_EXP_LOW` in complex/trig.rs
    `inverse_exp_sinh_cosh`; 106 sweep cells there, 3 miss):
    - S!B16 `IMCOSH("0.001258575663522")`: Excel "1.00000079200646", ours "1.00000079200645", math
      1.0000007920064549506. Excel is past the tie (0.495 of a unit) and ours matches the math.
    - S!A12 `IMSINH("-0.00878860151333914")`: Excel "-0.00878871465166476", ours "...477", math
      -0.0087887146516647654645. Excel is on the other side of a near tie.
    - S!A7 `IMSINH("0.00313307442770563-0.855889226338187i")`: real part Excel "0.00205388173561626", ours
      "...625", math 0.0020538817356162554411. Here Excel matches the math and ours does not.
  - **IMSIN/IMCOS with |y| > 20** (the EXP pair `exp_sinh_cosh`, and libm `x.sin()`/`x.cos()` for the real
    argument; 94 sweep cells there, 2 miss):
    - S!C117 `IMSIN("0.981793650815856-183.167307387565i")` and S!D117 `IMCOS(...)` of the same z: the part
      9.82267943792204E+78 in Excel's text, ours ...205E+78, math 9.822679437922043960E+78. Excel matches the math.
- So on 2 of the 3 hyperbolic cells Excel is not libm's sinh/cosh either. The forms already in trig.rs are libm, the
  `1/u` form and the EXP pair. Score each with the fit lab (`rustsheet-fitlab im-hyperbolic-fresh`, as T-3545 did) on
  the sweep's |x| < 0.01 cells and on the older probes there (probes/t2535-imsinh-imcosh-kernel-sweep;
  `IMSINH("0.01")` and `IMSINH("0.00012+1.23456789i")` are libm's digits). Do the same for the trig pair's real-argument
  SIN/COS (`excel_sin_cos` against libm) at |y| > 20.
- Any cut or form choice is structural (no tuned constant). It still needs decision 0188 R4's guards, as T-3545's had.
- Related: T-2797 (the lower switch between 0.01 and 0.0105), T-3546 (negative x near -709), T-3560 (notation).

## Visible-parity stage
Decision 0083: each of the 5 cells shows Excel's text, or is recorded in docs/excel-semantics/engineering.md as
UNVERIFIED with the candidate forms and their scores.

## Acceptance criteria
- [ ] probes-numeric/t3525-im-hyperbolic-fresh-sweep.case.json: S!A7, A12, B16, C117, D117 match Excel's text, or
      each is listed in engineering.md as above.
- [ ] Any kernel change keeps every other cell of the t3525 and t3545 sweeps and of the older IM* probes (fit lab
      `im-hyperbolic`, `im-hyperbolic-fresh`, `im-small-imag`): the counts before and after are in the Outcome.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188) for any changed cut or form:
      - hold-out counts per split, stated before scoring;
      - a fresh sweep drawn after the choice is frozen (|x| < 0.01 for the hyperbolic pair, |y| 20..709 for the trig
        pair), with its counts;
      - the Gate A envelope result on that sweep;
      - the `FITTED` tag and its range updated on any moved constant;
      - no step targets bits below the reproducibility floor;
      - no keep-matched branch.
- [ ] No gated known-failure line added (`make eval-baseline` refuses regressions); `make check` green; reviewers
      approve.

## Out of scope
- The 0.01..0.0105 switch itself (T-2797) and the log-space band (T-3546).
