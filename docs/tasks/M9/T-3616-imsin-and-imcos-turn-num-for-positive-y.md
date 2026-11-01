---
id: T-3616
title: "IMSIN and IMCOS turn #NUM! for positive y between 709.7 and 709.75, below exp's overflow at 709.78: pin and implement the cut"
milestone: M9
lane: functions-engineering
status: done
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
- Negative y is done (T-3589: `#NUM!` once EXP(y) is subnormal, below -708.3964185322641).

## Acceptance criteria
- [x] A probe brackets the positive cut to its double or to a stated constant; the rule is in engineering.md.
- [x] The 4 cells above match; no other IM* cell changes (fit lab `im-hyperbolic`, `im-hyperbolic-t3589-fresh`).
- [x] `make check` green; reviewers approve.

## Out of scope
- IMSINH/IMCOSH's own overflow (the log form, T-2231/T-3546).

## Outcome
probes-numeric/t3616-imsin-imcos-positive-cut-probe (330 cells, excel-graph) brackets the cut to the constant 709.7:
IMSIN and IMCOS answer at y = 709.7 and 709.699999999999 and are `#NUM!` from 709.700000000001 (the finest a
15-digit text writes) through 709.75, for all six real parts probed (0, 0.5, 1.2, -2.3, 3, 1.5707963267949).
EXP, COSH and SINH stay finite throughout, so this is BESSELI's 709.7 constant (T-2613), not an overflow. The
constant now lives once, as `ENGINEERING_EXP_CUT` in functions/engineering/mod.rs; BESSELI's `BESSELI_OVERFLOW_X` and
`trig_exp_overflows` (complex/trig.rs: `y > 709.7 || EXP(y)` subnormal) both use it. engineering.md has the new entry
"IMSIN and IMCOS positive overflow cut".

Eval delta: t3589-imsin-imcos-subnormal-exp-probe now passes 52/52 (it was 48/52; the 4 cells were the task's).
The new probe passes 320 of 330. The other 10 cells are not about the cut, and two follow-ups own them:
- T-3726 (opus, visible): the cos x part at x = 1.5707963267949 is 7E-6 relative off (K1, L1, K22, L22).
- T-3727 (opus, phase-1b): COSH and SINH are one ulp below Excel at 709.72, 709.7000001 and 709.700000000001.
probes-numeric is ungated (no baseline). No baseline changed in any suite, and the fit labs `im-hyperbolic*` are
unchanged.
