---
id: T-3545
title: "IMSIN and IMCOS sinh/cosh of a small imaginary part (|y| < 0.01): fit Excel's kernel"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IMSIN and IMCOS give Excel's digits for an imaginary part with |y| below the `1/u` band (|y| <= 0.01), where the
engine's `(EXP(t) ∓ EXP(-t))/2` pair (complex/trig.rs `exp_sinh_cosh` via `trig_sinh_cosh`) cancels.

## Context
- Found by T-3525's fresh sweep (probes-numeric/t3525-im-hyperbolic-fresh-sweep, excel-graph): 36 of the 39 misses
  are IMSIN (C) and IMCOS (D) cells with 1e-4 < |y| < 0.0085, e.g. `IMSIN("0+0.0080407127044978i")` is
  0.00804079934755902i in Excel, ...908i in the engine; `IMCOS("-0.437695624558513-0.000131316980692306i")` has
  imaginary part -0.0000556591552727565 (engine ...7094). The correctly rounded value misses most of them too (Excel's
  kernel is neither the double EXP pair nor exact), so Excel has its own small-t form there.
- Constraint: `IMSIN("1E-200+1E-200i")` is "1E-200" (sinh 0 from equal EXPs) and `IMCOS("150-0.002i")` matches the EXP
  pair (docs/excel-semantics/engineering.md, T-2535). An extended-precision EXP pair is one candidate to try first.
- Also C117/D117 (|y| = 183, above the band): one 15th-digit miss of the EXP pair.
- T-3525's Outcome and engineering.md "IMSIN and IMCOS of a small imaginary part are neither the EXP pair nor exact
  (T-3525)" have the counts.

## Acceptance criteria
- [x] A candidate kernel for |y| <= 0.01 is fitted through the fit lab (`rustsheet-fitlab im-hyperbolic-fresh` scores
      IM* text; decision 0188 R4: hold-out split, a fresh sweep after the freeze, `FITTED` tag if a constant is tuned).
- [x] The t3525 sweep's small-|y| IMSIN/IMCOS cells pass, `IMSIN("1E-200+1E-200i")` and the 1,023 older IMSIN/IMCOS
      cells keep passing (or a loss is listed within decision 0083's allowance).
- [x] `make check` green; reviewers approve.

## Out of scope
- IMSINH/IMCOSH below |x| = 0.01 (3 fresh-sweep misses; T-3525 keeps the band as is).

## Outcome
IMSIN and IMCOS now take sinh y and cosh y in the reciprocal form `(u - 1/u)/2`, `(u + 1/u)/2` with u = EXP(y)
for every |y| <= 20 (complex/trig.rs `trig_sinh_cosh`; the form split out as `inverse_exp_form`). Before, they used the
EXP pair below IMSINH's lower cut (0.01). No constant is tuned: the change drops a cut for IMSIN/IMCOS, a structural
choice among candidates, so it carries no `FITTED` tag (decision 0188 R4) but has a hold-out and a sweep after the freeze.

- **Candidates** on the 21 pure-imaginary small-|y| IMSIN cells of probes-numeric/t3525-im-hyperbolic-fresh-sweep (x = 0,
  so the part is sinh y; `docs/research/t3545/candidates.py`): `(EXP(y) - 1/EXP(y))/2` 21 (with idealized x87, libm or
  correctly rounded EXP alike), double EXP pair 13, `(1/EXP(-y) - EXP(-y))/2` 9, extended-precision EXP pair, libm sinh
  and Kahan-expm1 sinh 0. The winner is the form IMSIN/IMCOS already use inside the band (T-2796), so the model was:
  drop their lower cut.
- **Fresh sweep after the freeze** (R4.2): probes-numeric/t3545-imsin-imcos-small-imag-sweep (300 cells, |y|
  log-uniform in [1e-7, 0.0105], x = 0 or in (-1.5, 1.5); generator docs/research/t3545/gen_sweep.py, seed 3545;
  excel-graph). Fit lab `im-small-imag` (hold-out T-3360, margin 5 points):

| candidate | train exact / <=4 ulp / <=15 digits | test exact / <=4 ulp / <=15 digits | verdict |
|---|---|---|---|
| engine (reciprocal form) | 200 (100%) / 200 / 200 | 100 (100%) / 100 / 100 | accept |
| main (EXP pair below 0.01) | 150 (75.0%) / 150 / 150 | 77 (77.0%) / 77 / 77 | accept, below |
| correctly rounded | 55 (27.5%) / 55 / 55 | 24 (24.0%) / 24 / 24 | reference |

  Review asked for the rest of the changed domain, so a second sweep followed with the model unchanged:
  probes-numeric/t3545-imsin-imcos-tiny-imag-sweep (300 cells, |y| log-uniform in [1e-300, 1e-7], seed 35451,
  `gen_sweep.py --tiny`): engine 300/300 by text, main 297. Fit lab `im-small-imag` over both sweeps (600 cells):
  engine train 395/395, test 205/205 (accept); main 343/181; correctly rounded 100/54. So the form is evidenced for
  |y| from 1e-300 to 0.0105, and inside the band by T-2796 and T-3525.
- **Gate A envelope (R2)** on both sweeps: 600 of 600 inside, 0 outside (every cell equals Excel's value by digits;
  the fit lab's within-15-digits and within-1e-9 columns are 600/600).
- By text the harness counts 296/300 on the first sweep (main 224): the other 4 have equal digits and differ only in notation (14-digit
  parts at exponent -6 are plain in Excel), a separate number-to-text quirk, recorded in engineering.md and filed as
  T-3560 (A128, A43, B49, B81).
- **Old cells**: fit lab `im-hyperbolic` (1,359 IMSINH/IMCOSH/IMSIN/IMCOS cells of the T-2231/T-2535/T-2629/T-2778
  probes) 1,359/1,359 before and after; `im-hyperbolic-fresh` 1,151 -> 1,185 of 1,200 (train 815 -> 839, test 336 ->
  346). `IMSIN("1E-200+1E-200i")` stays "1E-200" and `IMCOS("150-0.002i")` keeps its digits.
- **Eval delta**: gated suites unchanged (`make eval-baseline` records nothing: no fix, no regression). probes-numeric
  (not gated): t3525 fresh sweep 961 -> 995 of 1,000; the new t3545 sweeps 296 of 300 and 300 of 300.
- IMSEC and IMCOT are unchanged (`sec_sinh_cosh` keeps libm below the band). Still open: `IMSIN`/`IMCOS("0.98…-183.17…i")`
  (|y| above the band) differ from the EXP pair in the 15th digit; 3 IMSINH/IMCOSH cells below |x| = 0.01.
- Follow-up: T-3560 (IM* part notation).

