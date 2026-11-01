---
id: T-3609
title: "IMSECH, IMCSCH and IMCOSH below |x| = 0.01: the 7 t3589 fresh-sweep texts one 15-digit digit off"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3589]
campaign: visible
---

## Goal
The 7 digit misses of probes-numeric/t3589-im-hyperbolic-small-x-sweep show Excel's text, or engineering.md records
them as UNVERIFIED with the forms tried and their scores.

## Context
- T-3589 froze, for |x| <= 0.01, sinh x as the ascending Taylor sum and cosh x as the `1/u` form
  (complex/trig.rs `inverse_exp_sinh_cosh`, `sinh_series`), then drew this sweep (decision 0188 R4.2). It fits all
  300 IMSINH texts and 149 of 150 IMCOSH by digits; the previous engine (libm) missed 15 of the 600 cells, the model 7:
  - IMSECH of a tiny real x (C42 `IMSECH("6.17632662504393E-06")` 0.999999999980927, ours ...926; C58, C68): Excel's
    `1/cosh x` is one ulp off `1/((1/u + u)/2)`. Over all 151 IMSECH texts with |x| <= 0.01 in the suites, the
    series sinh with libm's cosh fits all 151 (the frozen model 148), but that pairing misses 2 IMCOSH and 3 IMSINH
    texts the form fits; so IMSECH may compute its value another way (for example `2u/(1 + u²)` or `2/(1/u + u)`)
    rather than take a different cosh. Scores from T-3589's throwaway scorer over the suites' oracle texts.
  - IMCSCH D15, D29 (complex x near -0.009) and D102 `IMCSCH("-1.05118886127103E-06")` -951303.839721704 (ours ...703).
  - IMCOSH B15 `IMCOSH("-0.00908725305283191-0.897991007302082i")` imaginary part ...606 (ours ...607).
- The sweep is post-freeze data for T-3589's model; any new choice made on it needs its own fresh sweep (R4).
- Notation-only misses in the same sweep (14 digits at exponent -6 written plain) belong to T-3560.

## Acceptance criteria
- [ ] The 7 cells match Excel's text, or each is listed in docs/excel-semantics/engineering.md as UNVERIFIED with
      the candidate forms and their scores.
- [ ] Any kernel change keeps every cell of the fit lab's `im-hyperbolic`, `im-hyperbolic-fresh`, `im-small-imag`
      and `im-hyperbolic-t3589-fresh` fits, with before/after counts in the Outcome, plus the R4 guards
      (reviewer-excel-fidelity check 7).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Notation (T-3560); the 0.01 cut itself (T-2797).
