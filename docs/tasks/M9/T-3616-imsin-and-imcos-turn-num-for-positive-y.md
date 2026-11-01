---
id: T-3616
title: "IMSIN and IMCOS turn #NUM! for positive y between 709.7 and 709.75, below exp's overflow at 709.78: pin and implement the cut"
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
IMSIN and IMCOS give `#NUM!` exactly where Excel does for large positive imaginary parts.

## Context
- probes-numeric/t3589-imsin-imcos-subnormal-exp-probe (excel-graph, T-3589): `IMSIN("1.2+709.7i")` answers
  (7.71254900224736E+307+2.99848147982491E+307i), `IMSIN("0.5+709.75i")` and `IMSIN("-2.3+709.78i")` (and IMCOS of
  both) are `#NUM!`. Ours answers at 709.75 and 709.78: `trig_exp_overflows` (complex/trig.rs) tests libm `exp(|y|)`
  overflow, near ln(DBL_MAX) = 709.7827. These 4 cells are the case's only misses; main had them too.
- The cut lies in (709.7, 709.75]. BESSELI has a constant cut at 709.7 (engineering.md "BESSELI overflow point",
  T-2613); the IMSIN cut may be the same constant, or a product in Excel's form (`u = EXP(y)` with `u + 1/u`, or
  the parts' product with sin x) overflowing. Probe a fine bracket (709.70..709.75, several x, including x = 0)
  before choosing.
- T-3642's grid adds 32 cells in this band: IMSIN and IMCOS of x+709.78i are `#NUM!` in Excel for every x probed
  (0, 1E-308, 1E-300, 1E-200, 1E-20, 1, 20, 700, 705, 709, 709.5, 709.78, 710, 711, 715, 720), ours answers
  (probes/im-trig-magnitude-grid columns A and B, rows 12, 31, 50, 69, 88, 107, 126, 145, 164, 183, 202, 221, 240,
  259, 278, 297).
- Negative y is done (T-3589: `#NUM!` once EXP(y) is subnormal, below -708.3964185322641).

## Acceptance criteria
- [ ] A probe brackets the positive cut to its double or to a stated constant; the rule is in engineering.md.
- [ ] The 4 cells above match; no other IM* cell changes (fit lab `im-hyperbolic`, `im-hyperbolic-t3589-fresh`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSINH/IMCOSH's own overflow (the log form, T-2231/T-3546).
