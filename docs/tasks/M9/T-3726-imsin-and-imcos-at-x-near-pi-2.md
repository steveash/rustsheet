---
id: T-3726
title: "IMSIN and IMCOS at x near pi/2 with a large y: the cos x part 7E-6 relative off (t3616 K1, L1, K22, L22)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
IMSIN and IMCOS give Excel's text when the real part is near pi/2 and the imaginary part is large, so their
small part (cos x · sinh y for IMSIN, -cos x · cosh y for IMCOS's real part) matches Excel in all 15 digits.

## Context
probes-numeric/t3616-imsin-imcos-positive-cut-probe (excel-graph, T-3616), x = 1.5707963267949 (cos x is about
3.49E-15, so only a few digits of x past pi/2 survive):
- S!K1 `IMSIN("1.5707963267949+709.7i")`: Excel 8.27492013840132E+307-2.88915216855774E+293i, ours
  ...-2.88917292436455E+293i; K22 (y = 709.699999999999) the same shape.
- S!L1 `IMCOS("1.5707963267949+709.7i")`: Excel -2.88915216855774E+293-8.27492013840132E+307i, ours
  -2.88917292436455E+293...; L22 likewise.
Ours is 7.2E-6 relative off in the small part only; the large part matches. complex/trig.rs `imsin`/`imcos` take
`x.sin()`/`x.cos()` from libm. 7E-6 relative at a remainder of 3.5E-15 is an absolute error near 2.5E-20, the size of
a reduction with a 66-bit pi (x87 FSIN/FCOS), so Excel's IM* trigonometry may reduce x as the x87 does rather than as
SIN/COS do (`excel_sin_cos`, math_and_trigonometry/trig.rs, which IMSIN of a real number already uses). Check
which candidate (libm, `excel_sin_cos`, x87 66-bit-pi reduction) fits these 4 cells and keeps every other IM* cell,
then confirm on a fresh sweep of x near multiples of pi/2 with moderate and large y.
Needs an oracle sweep of x near k·pi/2 (k = 1..4, both signs) with |y| from 1 to 709: draw it after the
candidate is frozen.

## Acceptance criteria
- [ ] The 4 cells above match; no other IM* cell changes (fit labs `im-hyperbolic`, `im-hyperbolic-t3589-fresh`,
      `im-hyperbolic-log`, every IM* case in probes and probes-numeric).
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7): hold-out split fixed before scoring, a fresh sweep
      drawn after the freeze, Gate A envelope result, `FITTED` tag and range if a value is fitted, the reproducibility
      floor, and no keep-matched branch.
- [ ] The rule is in docs/excel-semantics/engineering.md.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of
Excel's value. Bit-level criteria still unmet then move to a follow-up task named in the Outcome.

## Out of scope
- The `#NUM!` cut at y > 709.7 (T-3616).
- COSH/SINH last bits near 709.7 (T-3727).
