---
id: T-2367
title: "Incomplete gamma ulp residue after T-0971: GAMMA.INV F38, POISSON.DIST D16 and the inversion cap shape"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0971]
defer: phase-1b
---

## Goal
The two cells T-0971's accurate incomplete gamma moved by one ulp are Excel's again, and the largest shape an
incomplete-gamma inversion finishes within the 2^19-term cap is measured and recorded.

## Context
T-0971 (`statistical/incomplete_gamma.rs` `incomplete_gamma_pair`) fixed 61 baseline cells and regressed two by one
ulp, under decision 0083's standing ulp allowance:
- `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA.DIST GAMMA.INV!F38` (GAMMA.INV): Excel 1, ours 0.9999999999999999.
- `ironcalc-calc/STATISTICAL/POISSON.xlsx` `POISSON.DIST!D16`: Excel 0.9863047314016171, ours 0.986304731401617.
- gen-m9 (after the merge with main's T-1634 `gamma_quantile`; all numeric, within 15 significant digits):
  `CHIINV(0.95,7)` (146b46e74f2dac3b A307) Excel 2.167349909298057, ours 2.167349909298058 (2 ulps);
  `GAMMA.INV(0.05,2,30)` (2801c1c11c21e134 A332) 10.660845320959863 vs 10.660845320959861 (1);
  `POISSON(3,1.5,-1)` (38e4fc309fe199ea A57, c7664e494e418395 A256) and `CHIDIST(3,8)` (f2bd5f380ec002bf A174)
  0.9343575456215498 vs 0.9343575456215499 (1); `CHIDIST(3,4)` (530c636749a85209 A242, a612e89321a6df9c A273)
  0.5578254003710746 vs 0.5578254003710748 (1); `CHIINV(Data!B4,4)` (899f9555e67da2d6 A5) 4.878432966560409 vs
  4.8784329665604105 (2); `CHISQ.INV.RT(0.1,7)` (984cba67bbe6972b A216) 12.01703662378053 vs 12.017036623780527 (2);
  `CHISQ.INV(Data!E1,...)` (a61e547a1cab994c A1) 0.015790774093431225 vs 0.015790774093431232 (2);
  `CHIDIST(LN(7),9)` (a7bf0d8d81d33ba6 A121) 0.9922943358901939 vs 0.9922943358901938 (1); the `CHISQ.INV.RT` in
  d0854366830dcdee A238 5.4441969945493144E-15 vs 5.444196994549313E-15 (2); `CHIDIST(...,10)`
  (fd9fa22d212080d6 A123) 0.9908757207816048 vs 0.9908757207816047 (1).
Ungated probes-numeric cells newly 1 ulp off with T-0971:
- m9-chisq-inv-rt A40, m9-chisq-inv A38;
- m9-gamma-dist-a0p5 A22, m9-gamma-dist-a1 A24 and A26, m9-gamma-dist-a2 A2 and A12;
- m9-gamma-inv A31, A32 and A38;
- t1048-gammainv A1, A15, B1 and B15.

Also probes-numeric/m9-chisq-dist-rt A14 `CHISQ.DIST.RT(6,3)`. It moved from 8 to 19 ulp off Excel's
0.11161022509471255, which is 1 ulp from the 50-digit value, while ours is 18 ulp from it. That makes it the one
cdf cell left where Excel is accurate and ours is not, at a = 1.5 (the continued fraction or prefactor). Add
a = 1.5 to the mpmath unit-test grid.

The series now runs to 2^-54 instead of statrs' 1E-15 (about 8% more terms near x = a), so an inversion's shared
budget runs out at a smaller shape than statrs' 3.5E8 (T-1872's 3-million-sample sweep): `GAMMA.INV(1E-5,7.9E8,1E-6)`
is now `#NUM!` (530,000 terms over its cdfs), shape 3E8 still finishes. docs/excel-semantics/statistics.md
"Incomplete gamma: accurate kernel (T-0971)" and "Incomplete gamma: per-call cap and lifting (T-1872)".

## Acceptance criteria
- [ ] GAMMA.xlsx F38, POISSON.xlsx D16 and the 13 gen-m9 cells above pass; no new baseline lines.
- [ ] T-1872's inversion sweep re-run on the new kernel; the smallest shape past the cap is recorded in the cap
      section (and `MAX_TERMS`'s doc), or a faster large-shape method (Temme's expansion) keeps 3.5E8.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).

## Phase 1 triage (T-0090, 2026-10-27)
Two visibly-off gen-m9 cells matched this parked task: IRR gen-m9/d0854366830dcdee Calc!A216 is handed to T-2131 and FREQUENCY gen-m9/fd9fa22d212080d6 Calc!A32 (an error-kind cell) to T-2636.
