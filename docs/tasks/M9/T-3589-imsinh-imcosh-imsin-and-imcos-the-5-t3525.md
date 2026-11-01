---
id: T-3589
title: "IMSINH, IMCOSH, IMSIN and IMCOS: the 5 t3525 fresh-sweep texts one 15-digit tie off (|x| < 0.01 in the hyperbolic pair, |y| > 20 in the trig pair)"
milestone: M9
lane: functions-engineering
status: done
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
- [x] probes-numeric/t3525-im-hyperbolic-fresh-sweep.case.json: S!A7, A12, B16, C117, D117 match Excel's text, or
      each is listed in engineering.md as above.
- [x] Any kernel change keeps every other cell of the t3525 and t3545 sweeps and of the older IM* probes (fit lab
      `im-hyperbolic`, `im-hyperbolic-fresh`, `im-small-imag`): the counts before and after are in the Outcome.
- [x] R4 guards (reviewer-excel-fidelity check 7, decision 0188) for any changed cut or form:
      - hold-out counts per split, stated before scoring;
      - a fresh sweep drawn after the choice is frozen (|x| < 0.01 for the hyperbolic pair, |y| 20..709 for the trig
        pair), with its counts;
      - the Gate A envelope result on that sweep;
      - the `FITTED` tag and its range updated on any moved constant;
      - no step targets bits below the reproducibility floor;
      - no keep-matched branch.
- [x] No gated known-failure line added (`make eval-baseline` refuses regressions); `make check` green; reviewers
      approve.

## Out of scope
- The 0.01..0.0105 switch itself (T-2797) and the log-space band (T-3546).

## Outcome
- **Model** (complex/trig.rs): at |x| <= 0.01 `inverse_exp_sinh_cosh` takes sinh x as the ascending Taylor sum
  (`sinh_series`: doubles, each term the one before times x²/((2k)(2k+1)), until a term leaves the sum unchanged)
  and cosh x as the `1/u` form; IMSIN and IMCOS (`trig_sinh_cosh`) take the `1/u` form for every |y|, the EXP pair
  past 20 removed (`exp_sinh_cosh` deleted). For y < -20 that makes cosh y `1/EXP(y)`, not `EXP(-y)`. Structural
  choices, no constant moved (`INVERSE_EXP_LOW`/`HIGH` keep their T-3525 `FITTED` tags and brackets).
- **The 5 cells**: S!A7, A12, B16, C117, D117 now show Excel's text; t3525-im-hyperbolic-fresh-sweep 995 → 1000/1000.
- **Candidates** (scored by a throwaway scorer over every oracle text in the suites: 114 IMSINH/IMCOSH texts with
  0 < |x| <= 0.01, 97 IMSIN/IMCOS with |y| > 20): libm 111; the `1/u` form for both 25; worksheet SINH/COSH (expm1)
  111-112; Horner series 112; ascending series + `1/u` cosh 114 (+ EXP-pair cosh 114, + libm cosh 113). Past 20:
  EXP pair 95, libm 95, Excel SIN/COS 95, double-double 91, `1/u` form 97. Recorded in engineering.md.
- **R4 guards** (decision 0188):
  - hold-out split (R4.1): the fit lab's fixed split (sha256 of `rustsheet-fitlab-holdout/v1`, case and cell; 30%
    held out), set before any scoring; accept = test exact rate at most 5.0% below train and above the correctly
    rounded reference on test. Exact / within 4 ulp / within 15 digits:

    | fit | candidate | train | test | verdict |
    |---|---|---|---|---|
    | `im-hyperbolic-fresh` (the t3525 cells) | model | 850/850 / 850 / 850 | 350/350 / 350 / 350 | accept |
    | | main | 847/850 / 847 / 847 | 348/350 / 348 / 348 | accept |
    | | correctly rounded | 603/850 / 603 / 603 | 255/350 / 255 / 255 | reference |
    | `im-hyperbolic-t3589-fresh` (post-freeze) | model | 438/439 / 438 / 438 | 161/161 / 161 / 161 | accept |
    | | main | 430/439 / 430 / 430 | 157/161 / 157 / 157 | accept |
    | | correctly rounded | 422/439 / 422 / 422 | 156/161 / 156 / 156 | reference |
    | `im-hyperbolic` (older cells) | model = main | 965/965 | 394/394 | accept (reference 741, 291) |
    | `im-small-imag` | model = main | 395/395 | 205/205 | accept (reference 100, 54) |

    `im-hyperbolic-log` (891/891) and `im-hyperbolic-log-fresh` (300/300) are unchanged.
  - fresh sweep after the freeze (docs/research/t3589/gen_sweep.py, excel-graph): t3589-im-hyperbolic-small-x-sweep
    (600 cells, |x| in [1e-6, 0.01], IMSINH/IMCOSH/IMSECH/IMCSCH) and t3589-imsin-imcos-large-imag-sweep (300 cells,
    |y| in [20, 709.7], both signs). Fit lab `im-hyperbolic-t3589-fresh` (new; the 600 IMSINH/IMCOSH/IMSIN/IMCOS
    cells): model 599 (train 438/439, test 161/161), main 587, correctly rounded 578. Large-|y| case 300/300 (main 296).
    By digits over all six functions on the small-x case: 7 misses, main 15 (IMSINH 0/5, IMCOSH 1/4, IMCSCH 3/4,
    IMSECH 3/2); the rest are notation-only (T-3560). Follow-ups: T-3609 (the 7), T-3616 (positive-y cut).
  - Gate A envelope (docs/research/t3589/gate_a.py): 900/900 fresh cells within 1e-9 of the 50-digit value.
  - `FITTED`: no constant moved. Reproducibility floor: each choice is between forms that differ by whole ulps on
    Excel's printed 15 digits, never below. No keep-matched branch.
- **Review round 1** (excel-fidelity): probed IMSIN and IMCOS at |y| in [708.4, 709.78], where `1/EXP(y)` starts
  from a subnormal EXP. Excel is `#NUM!` for every y < ln(DBL_MIN) = -708.3964185322641 (cut bracketed between
  -708.39641853, answers, and -708.396419, `#NUM!`): its EXP flushes to 0 and the reciprocal overflows, which
  confirms the lopsided form. Implemented in `trig_exp_overflows` (no constant: the normal-range test of EXP(y)).
  t3589-imsin-imcos-negative-cut-probe 22/22, t3589-imsin-imcos-subnormal-exp-probe 48/52; its 4 misses are
  y = +709.75 and +709.78 (`#NUM!` in Excel, answered by ours and by main), filed as T-3616.
- **Eval delta**: probes-numeric +5 (t3525 fresh sweep 1000/1000) and four new cases (the two sweeps 882/900 by
  text: 11 notation, 7 digits; the two review probes 70/74); gated suites unchanged (`make eval-baseline`: 0 fixed, 0 regressions; probes-numeric is not gated).
