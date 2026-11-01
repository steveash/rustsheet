---
id: T-3546
title: "IMSINH and IMCOSH of a negative real part: pin where the log form starts (below |x| = 708.94)"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IMSINH and IMCOSH of x + yi with x < 0 switch to the log form (complex/trig.rs `hyperbolic_log_form`) where Excel
does, which is below |x| = 708.94, not at |x| > 709 as for x > 0; and the log form's digits for x < 0 match.

## Context
- Found by T-3525's log-band sweep (probes-numeric/t3525-im-hyperbolic-log-band-sweep, excel-graph): with x < 0,
  `IMSINH("-708.993723853194-0.284042075249253i")` and `IMCOSH` of it are `#NUM!` (sin y < 0: the log form's
  signature), `"-708.952568260886+1.74023049176272i"` too (cos y < 0), and `IMSINH("-708.931415280617+0.631619003955773i")`
  is -3.09659439784082E+307+2.26546023028534E+307i, the log form's digits (product form: ...096E+307+...528E+307i).
  For x > 0, `"708.959727…+1.343…i"` keeps the product form and the T-2231 bracket (709, 709.0000000001] stands.
  8 of the sweep's 10 engine misses are these; the other 2 are `IMSINH`/`IMCOSH("-709.183834084079+1.0222540398548i")`,
  one 15th-digit miss of the log form's imaginary part at x < 0.
- The rule may depend on y, not on a single |x| cut: S!A84 (x = -708.931) has the log form's digits, S!A38
  (x = -708.9937) is `#NUM!`, while S!A71 (x = -708.9941, both factors positive) is a number that differs from the
  math in the 13th digit (docs/research/t3525/gate_a.tsv). Design the probe to separate x from y.
- One hypothesis to probe (not evidence): the negative side cuts where EXP(x) leaves the normal range
  (ln DBL_MIN = -708.3964).
- docs/excel-semantics/engineering.md "IMCOSH and IMSINH past |x| = 709 work in log space (T-2231)" has the notes.

## Acceptance criteria
- [x] A probe brackets the negative-side cut to 1e-10 (excel-graph), and the cut is implemented from that bracket
      (an evidenced constant, not a fit), with engineering.md updated.
- [x] The 8 cut cells of the t3525 log-band sweep pass; no older IMSINH/IMCOSH cell regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 1/u band (T-3525 keeps it).

## Outcome
- Cut: probes-numeric/t3546-imsinh-neg-cut-coarse (x = -709.2..-708 step 0.01, four columns: IMSINH(x-0.5i),
  IMCOSH(x+2i), IMSINH(x-1E-200i), IMSINH(x+1i)) and t3546-imsinh-neg-cut-fine (x = -708 - d, d = 1E-12..9E-3)
  put the negative-side cut between -708.000000000001 and -708 (bracket 1e-12), the same for every y tried, so it is
  one constant, not a y rule: `HYPERBOLIC_LOG_CUT_NEGATIVE = -708` (evidenced bracket, not a fit). The ln DBL_MIN
  hypothesis (-708.3964) is refuted.
- Digits: the log form now takes Excel's EXP (`excel_exp`) instead of libm `exp`. Structural model choice from two
  candidates (decision 0188 R4, check 7 kind (c), no `FITTED` tag: nothing is fitted). The libm build is
  docs/research/t3546/libm-exp.patch. Fit lab `im-hyperbolic-log` (891 cells: eng-im-overflow-edges(-2),
  t3525-im-hyperbolic-log-band-sweep, t3546 cut probes; hold-out T-3360, 30% by hash, margin 5 points fixed by the lab
  before scoring):

| candidate | train exact / <=4 ulp / <=15 digits | test exact / <=4 ulp / <=15 digits | verdict |
|---|---|---|---|
| engine (Excel's EXP) | 650 (100%) / 650 / 650 | 241 (100%) / 241 / 241 | accept |
| libm `exp` build | 646 (99.4%) / 646 / 646 | 240 (99.6%) / 240 / 240 | accept, below |
| correctly rounded | 51 (7.8%) / 51 / 51 | 24 (10.0%) / 24 / 24 | reference |

- Fresh sweep after the freeze (R4.2): probes-numeric/t3546-im-hyperbolic-log-exp-fresh-sweep (150 rows x 2, 76
  `#NUM!`; docs/research/t3546/gen_sweep.py, random.Random(35461): x < 0 with |x| uniform in [708, 710.5], x > 0 in
  [709, 710.5]; uniform rather than log-uniform, which over this 0.4%-wide |x| window is the same distribution to
  within 0.2%). Fit lab `im-hyperbolic-log-fresh`: engine train 214/214, test 86/86 (accept); libm 209/85 (max 50 ulp);
  correctly rounded 2/0.
- Gate A envelope (R2) on the fresh sweep: 300 of 300 inside, 0 outside (every cell equals Excel's result by text,
  the 76 `#NUM!` cells included, so R2's equal-to-Excel branch; 224 of the 224 numbers are also within 1e-9 of the math).
- Not changed: the log form keeps libm `sin`/`cos` of y (UNVERIFIED in engineering.md for a large y; within
  |y| < 3.1 every probe cell matches, so no follow-up is filed until a large-y probe shows a difference).
- Eval (probes-numeric, ungated, no baseline): t3525-im-hyperbolic-log-band-sweep 190 -> 200/200 (the 8 cut cells and the 2 digit cells); new cases
  t3546-* 868/868; eng-im-overflow-edges(-2) unchanged at 116/116, 114/114. engineering.md updated.
