---
id: T-2435
title: "Scalar distributions at extreme arguments: BINOM.DIST, BETAINV, GAMMA.DIST, CRITBINOM, FDIST and LOGNORMDIST range checks (gen-m9, 20 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
BINOM.DIST, BINOMDIST, BINOM.INV, CRITBINOM, BETAINV, GAMMA.DIST, GAMMADIST, FDIST, LOGNORMDIST and EXPON.DIST give Excel's
result at extreme arguments: `#NUM!` for p = 1 with fewer successes than trials, a slightly negative number_s, 2^32 trials,
a tiny beta alpha, a huge gamma shape, df 1E10, 1E15 trials, and 0 for a pdf that underflows or is subnormal.

## Context
Split 2026-10-26 (T-2435 implementation): the GAMMA.DIST huge-shape cells (2801c1c11c21e134 A241,
40425341c5034eed A265) moved to T-1891, and the BETAINV underflow and F.INV.RT far-tail cells
(3e98b26cf6713e77 A151, abc2939ebd1e2365 A11, a612e89321a6df9c A23, 01523d247594952e A377) to
T-2808: both need Excel's own search internals, which this task's five argument-range rules do not.
The counts below are from before that split.

Split 2026-10-25 (gap triage, sonnet refill): the slightly negative number_s cells (85f48d5cdfb24555 A35, caa1eed48b1883fe A384, 7bd219ec0c0c5e22 A320) moved to T-2700, which checks the sign of every discrete distribution's counts before truncation. The counts below exclude them.

Gap triage of the T-2255 gen-m9 sweep. statistics.md "Scalar distributions at extreme arguments (T-2255)" lists each cell;
20 cells, one per shape, so each rule rests on one cell. Before implementing a threshold (where 2^32 trials or a 1E15
CRITBINOM starts to be `#NUM!`, how small a beta alpha), probe a few values around it (excel-probe skill). EXPON.DIST's
0 is numbers.md "Subnormal results become 0" applied to a result. Overlaps: T-1891 (huge gamma shapes), T-1871 (BETAINV
tails), T-1538 (huge df); take the cell from them if one lands first.
Code: functions/statistical/binom.rs, beta.rs, gamma.rs, fisher.rs, lognormal and exponential readers.

Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 17 known-failure lines in eval/baseline/gen-m9.txt, 17 formulas (20 before the T-2700 split).
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A344 `=BINOM.DIST(0.1,2,1,0)`: Excel #NUM!, ours 0.0
- gen-m9/165db58d1540f014 Calc!A65 `=EXPON.DIST(ERROR.TYPE(MATCH("TRUE",Data!C5:E6,-1)),(100+MIN(Data!C8,Data!D7:D7,2)),0)`: Excel 0.0, ours 8.36257629135218e-309
- gen-m9/2801c1c11c21e134 Calc!A99 `=GAMMA.DIST(1E+307,100,0.5,FALSE)`: Excel 0.0, ours #NUM!
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A344
- gen-m9/165db58d1540f014.case.json (1): Calc!A65
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A99, A241
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A151
- gen-m9/40425341c5034eed.case.json (1): Calc!A265
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A104
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A125
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A240
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A97, A206 (A97, `LOGNORMDIST(..,MDURATION(..,1E+300,2))`, passes since T-2488: its MDURATION is `#NUM!`)
- gen-m9/a5828d3925575713.case.json (1): Calc!A315
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A127
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A231
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A56
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A363
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A345

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): more scalar distributions at extreme arguments: `PHI(1E+300)`, `NORM.S.DIST(1E+300,FALSE)` and `NORM.S.DIST(-1E+300,<FALSE from a formula>)` are `#NUM!` (ours 0), `BETA.DIST(0,0.5,1E+5,FALSE)` `#NUM!` (ours 0), `WEIBULL(100,1E+8,99999,..)` 0 and `WEIBULL(0.2,1E+15,0.2,"TRUE")` 0.6321205588285577 (ours `#NUM!` for the huge alpha), `F.INV.RT(1E-200,1,1.999)` 3.696066238512854E+150 (ours `#NUM!`) and `F.INV.RT(1E-20,3,9)` a value whose GAMMALN.PRECISE is 1087085.7102783846 (ours ...853, the 15th digit).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 8 known-failure lines in eval/baseline/gen-m9.txt, 8 formulas.
Examples:
- gen-m9/165db58d1540f014 Calc!A45 `=KURT(PHI(1E+300))`: Excel #NUM!, ours #DIV/0!; inner `PHI(1E+300)` Excel #NUM!, ours 0.0
- gen-m9/35e8168aae2b00f8 Calc!A48 `=AVERAGEA(BETA.DIST(0,0.5,1E+5,FALSE))`: Excel #NUM!, ours 0.0; inner `BETA.DIST(0,0.5,1E+5,FALSE)` Excel #NUM!, ours 0.0
- gen-m9/4ab2ba98d26c8a5e Calc!A341 `=FDIST(NORM.S.DIST((-1E+300),(Data!$D1=2)),2,COUNTIF(Data!C2:E4,""))`: Excel #NUM!, ours 1.0; inner `NORM.S.DIST((-1E+300),(Data!$D1=2))` Excel #NUM!, ours 0.0
- gen-m9/8dd2312c96effb1b Calc!A228 `=LARGE((7-SUBTOTAL(2,Data!D5:E12)),(NORM.S.DIST(1E+300,FALSE)-IFNA(#NULL!,Data!$B$1)))`: Excel #NUM!, ours #NULL!; inner `NORM.S.DIST(1E+300,FALSE)` Excel #NUM!, ours 0.0
Cells:
- gen-m9/165db58d1540f014.case.json (1): Calc!A45
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A48
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A341
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A228
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A276
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A250, A11
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A23

T-2496 probe (probes/t2496-fin-subexpressions, each case's Data sheet copied as Dnn, every call and compound argument read through a TYPE/shape summary, excel-graph, 2026-10-25; probes/tbill-error-order-inner-calls for the T-bill cells): the innermost diverging sub-expression of 1 gen-m9 financial cell moved here from T-2496.

| gen-m9 cell | innermost sub-expression | Excel | ours |
|---|---|---|---|
| gen-m9/d0854366830dcdee Calc!A298 | `WEIBULL.DIST(7,1E+8,Data!D5,"TRUE")` (beta 0.3: (x/beta)^alpha overflows) | #NUM! | 1 |

T-2491 hand-off (2026-10-25): gen-m9/01523d247594952e Calc!A377 `=SYD(15023,(AVEDEV(Data!A11:A11,{1;2;3;4;5})*F.INV.RT(1E-20,9,3)),33,1)`
is Excel -1987062826671.328, ours -1987062826671.3352. SYD's operation order cannot produce a 30-ulp gap, so the salvage must be
about 30 ulp off: the cause is F.INV.RT's far tail (`F.INV.RT(1E-20,9,3)`). This task owns that cell.

## Acceptance criteria
- [x] Visible parity (gen-m9): the 9 known-failure lines this task kept after the 2026-10-26 split
      disappear from eval/baseline/gen-m9.txt (`make eval-baseline`): 05d44f6dd6cfc567 A344,
      a013a45a8fbcab4f A206, a61e547a1cab994c A127, ee2497a5e683c5d9 A345, 165db58d1540f014 A45,
      35e8168aae2b00f8 A48, 4ab2ba98d26c8a5e A341, 8dd2312c96effb1b A228, c6cfda6bd10cae93 A363.
      The 6 cells of the split are T-1891's and T-2808's; the T-2496 and T-2491 blocks above are
      T-2808's (01523d247594952e A377) and already passing (d0854366830dcdee A298).
- [x] Each rule's probe cells are in eval/ (probes/t2435-dist-extremes, 140 formulas with a `(x)=0`
      companion each), and statistics.md gets an entry per rule with an "Ours:" line naming this task.
- [x] No new known-failure lines in gen-m9 or any other baseline; every new line in
      eval/baseline/probes.txt is named in an owning task file: T-1891 (A21, A22, A25, A26, A33,
      A34, A101, A102), T-2808 (A68, A69, A72, A73, A77:A81, A108:A112, A115:A118), T-2809 (A15,
      A18) and T-2811 (the last-digit cells A14, A20, A23, A24, A32, A35:A38, A40, A41, A66).
- [x] `make check` green; reviewers approve.

## Out of scope
- Last digits of these functions (their accuracy tasks).
- GAMMA.DIST at a huge shape (T-1891), BETAINV's underflow and F.INV.RT's far tail (T-2808).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probed the sweep's extremes on excel-graph (probes/t2435-dist-extremes, 140 formulas and their
`(x)=0` companions, four fills) and implemented the five rules the evidence settles, each with its
own statistics.md entry:
- BINOM.DIST's and BINOMDIST's pmf at k = 0 with p = 1 is `#NUM!` from one trial on (Excel's
  k = 0 closed form takes LN(0)), while the cdf there stays 0 and 0 trials give 1.
- BINOM.INV and CRITBINOM take BINOM.DIST's 2^31 - 1 trial limit (`TRIALS_LIMIT`).
- NORM.DIST, NORMDIST, NORM.S.DIST and PHI report `#NUM!` for a density whose standardized value
  squares past the double range (|z| >= 2^512), not the limit 0 just below it
  (`special::normal_density_overflows`, now shared by the three call sites).
- BETA.DIST's density is `#NUM!` at an x of exactly 0 with alpha <= 1 and at an x of exactly 1 with
  beta <= 1 — against the given x, not the [A, B] ends, so it fires at interior points of a shifted
  interval, while at that interval's own ends, where the density diverges as much, Excel gives 0.
- Every F cdf is `#NUM!` where df2/(df1 x) overflows (`fisher.rs::f_beta_ratio_overflows`), where
  the density there is 0.
Eval delta: gen-m9 -10 known failures: the 9 cells of the acceptance criterion plus
abc2939ebd1e2365 A187 `=GROWTH(PHI((-1E+300)),..)`, which the PHI rule fixes too. No other suite
regressed; probes gained the new case's 40 known failures, each named in its owning task.
Follow-ups: T-2808 (F.INV.RT saturation, BETAINV underflow and its 0.5 quirk, 4 gen-m9 cells),
T-2809 (BINOM.INV's count at a huge trial count), T-2811 (the probe's last-digit cells), and the
GAMMA.DIST huge-shape cells added to T-1891. The probe's cells for those four stay known failures
in eval/baseline/probes.txt, each named in the owning task.
Round 2 (excel-fidelity review): the probe found that our density errored at a shifted interval's
own end where Excel gives 0 (A126, A127) — fixed here rather than recorded as a gap; the F cut's
lower side now has a normal-x cell either side (A85 against A141, A142), since `FDIST(1E-308,6,1E+8)`
is 1 only because a subnormal x reaches the kernel as 0.
