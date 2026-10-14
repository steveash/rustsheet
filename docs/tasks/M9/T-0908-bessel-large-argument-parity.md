---
id: T-0908
title: "BESSELJ/BESSELY parity for |x| >= 8"
milestone: M9
lane: functions-engineering
status: todo
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
- [ ] Identify Excel's large-argument algorithm from the cached values (fit candidate asymptotic
  expansions; use the oracle for more points via `excel-probe` if available).
- [ ] Remaining BESSELJ/BESSELY known failures removed from `eval/baseline/ironcalc-calc.txt`.
- [ ] `docs/excel-semantics/engineering.md` updated.
