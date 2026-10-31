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

T-3131 (2026-10-31, unit test `a_call_past_its_cap_is_nan`): `gamma_quantile(1E7, 1, 0.5)` and `(3E7, 1E-6, 1E-5)`
are already NaN on main 1248554ae, before and after T-3131's Wilson-Hilferty start, so the cap binds well below 3E8
now. The start uses the lower tail's probability, so for a tiny upper-tail p (`CHISQ.INV.RT(1E-300,df)`, the lower tail
rounding to 1) it is NaN and the refinement halves the whole range, which can spend the budget sooner from about
shape 2E6 to 1E9 (statrs' search also took the lower tail). The re-run sweep should cover both tails.

Gap triage 2026-10-31c (`make eval` on origin/main 0c917f9fe, excel-graph): 20 more POISSON and POISSON.DIST cdf
cells, 1 or 2 ulp off and within 15 significant digits, that no open task listed. All have a mean below about
4(k + 1) (mostly the lower-tail P near 1 at k above the mean, and the centre at k = mean = 100), so not the deep upper
tail T-3269 owns. Examples, Excel vs ours:
`POISSON(7,0.999,TRUE)` 0.9999898235766547 vs 0.9999898235766546 (0bbab6206b5c9a07 A91); `POISSON.DIST(6,0.999,TRUE)`
0.9999172685186382 vs 0.999917268518638 (a2262ea466fcc142 A202, bcb1105c61a2649d A79, 329b177b5d5165ad A99);
`POISSON.DIST(4,7,TRUE)` (787ba8d801105a60 A393, mean ROUNDUP(INT(7),7)) 0.17299160788207132 vs 0.17299160788207138 (+2);
`POISSON.DIST(100,100,TRUE)` 0.5265621985299984 vs 0.5265621985299986 (ede0323c389fc703 A371, +2);
`POISSON.DIST(1,1,TRUE)` 0.7357588823428848 vs 0.7357588823428847 (c4635963fc926284 A216).
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A91
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A366
- gen-m9/329b177b5d5165ad.case.json (2): Calc!A99, A306
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A335
- gen-m9/44a8841352dc48e9.case.json (1): Calc!A183
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A240
- gen-m9/787ba8d801105a60.case.json (2): Calc!A83, A393
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A384
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A116
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A202
- gen-m9/a4090567758520df.case.json (1): Calc!A378
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A107
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A79
- gen-m9/c4635963fc926284.case.json (1): Calc!A216
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A311
- gen-m9/d8f44d556674c2fd.case.json (1): Calc!A343
- gen-m9/ede0323c389fc703.case.json (1): Calc!A371
- gen-m9/f82473275380882e.case.json (1): Calc!A213

Gap triage 2026-10-31e (`make eval` on origin/main 4ff408243, excel-graph and cached probes): 29 more
CHISQ.DIST.RT, CHIDIST and CHISQ.DIST cdf cells on the ungated probes-numeric cases, 1 to 3 ulp off and within 15
significant digits, that no open task listed. Many sit at the small half-integer shapes of T-3285 (accuracy). If its
kernel fix reaches them, they pass there; the rest stay here. Examples, Excel vs ours:
`CHISQ.DIST.RT(1.5,3)` (m9-chisq-dist-rt A12) 0.6822703303362125 vs 0.6822703303362126;
`CHISQ.DIST.RT(0.1,5)` (t1049-chisqdistrt A3, m9-legacy-chidist A22) 0.9998376833880773 vs 0.9998376833880774;
`CHISQ.DIST(50,100,TRUE)` (m9-chisq-dist A63) 6.953305247616096E-06 vs 6.953305247616095E-06.
- probes-numeric/m9-legacy-chidist.case.json (12): Calc!A21, A22, A29, A30, A37, A38, A53, A54, A55, A56, A63, A64
- probes-numeric/m9-chisq-dist-rt.case.json (11): Calc!A12, A13, A15, A20, A24, A25, A27, A29, A30, A33, A34
- probes-numeric/t1049-chisqdistrt.case.json (5): Calc!A3, A11, A15, A23, A24
- probes-numeric/m9-chisq-dist.case.json (1): Calc!A63

## Acceptance criteria
- [ ] GAMMA.xlsx F38, POISSON.xlsx D16 and the 13 gen-m9 cells above pass; no new baseline lines.
- [ ] The 20 POISSON cdf cells added by gap triage 2026-10-31c leave eval/baseline/gen-m9.txt, or the Outcome lists each still failing with its ulp distance.
- [ ] The 29 probes-numeric cdf cells added by gap triage 2026-10-31e pass bit-exact in `make eval`'s report.json
      (probes-numeric is ungated), or the Outcome lists each still failing with its ulp distance.
- [ ] T-1872's inversion sweep re-run on the new kernel; the smallest shape past the cap is recorded in the cap
      section (and `MAX_TERMS`'s doc), or a faster large-shape method (Temme's expansion) keeps 3.5E8.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).

## Phase 1 triage (T-0090, 2026-10-27)
Two visibly-off gen-m9 cells matched this parked task: IRR gen-m9/d0854366830dcdee Calc!A216 is handed to T-2131 and FREQUENCY gen-m9/fd9fa22d212080d6 Calc!A32 (an error-kind cell) to T-2636.
