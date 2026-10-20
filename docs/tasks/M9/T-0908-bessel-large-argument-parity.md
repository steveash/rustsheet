---
id: T-0908
title: "BESSELJ/BESSELY parity for |x| >= 8"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: M
tier: opus
depends_on: [T-0900, T-1004]
---

## Goal
T-0900 made `|x| < 8` bit-exact. The remaining BESSELJ/BESSELY failures in
`ironcalc-calc/ENGINEERING/BESSEL.xlsx` are x = ±10 and ±32 (Excel is ~1e-6..1e-10 off the true
values, differing from fdlibm and from NR's asymptotic branch).

## Notes
Kernel map (T-1522, decision 0056): rank 9 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Once settled, drop the fdlibm fallback for |x| >= 8 in `bessel_excel.rs` if Excel never uses it.

Also probe Excel's real cap on the order `n` (currently UNVERIFIED: > 100_000 returns #NUM!).

**Needs an oracle sweep (T-0018):** fitting this formulation needs more Excel points than the
cached workbooks give. Fit against the T-1004 sweep (`eval/suites/probes-numeric/cases/m9-bessel.case.json`, filled), never blind.

## Acceptance criteria
- [x] Identify Excel's large-argument algorithm from the cached values (fit candidate asymptotic
  expansions; use the oracle for more points via `excel-probe` if available).
- [x] Remaining BESSELJ/BESSELY known failures removed from `eval/baseline/ironcalc-calc.txt`.
- [x] `docs/excel-semantics/engineering.md` updated.

## Outcome
Excel's `|x| >= 8` branch is the Numerical Recipes asymptotic form (`sqrt(0.636619772/x)`, phase shifts
0.785398164 and 2.356194491), but its coefficient tables have three differences from the book. They were
found by least squares over the m9-bessel sweep (14 x values per function) and then confirmed bit-exact:
J0's `P` uses `-0.1098628267e-2` (the book has `…627e-2`), Y0's `Q` uses `-0.934945152e-7` (the book has
`…935…`), and J1's `P` repeats `-0.3516396496e-4`. J's upward recurrence also needs the `b * (2j * (1/x))`
association that Y already used. `bessel_excel.rs` now covers every x, and the fdlibm port
(`bessel_j0_y0.rs`, `bessel_j1_y1.rs`, `bessel_jn_yn.rs`, `bessel_util.rs`, about 1,140 lines) and its
NOTICE line are gone.
- Eval: ironcalc-calc known failures 1008 → 982 (26 fixed; `ENGINEERING/BESSEL.xlsx` 476/476), 0 regressions.
- probes-numeric/m9-bessel (not gated): 362/364 bit-exact, up from 14. The other 2 are `BESSELJ(10|50,1000000)`,
  the documented order-cap DEVIATION.
- Excel's real order cap was not probed. It stays the existing DEVIATION.
