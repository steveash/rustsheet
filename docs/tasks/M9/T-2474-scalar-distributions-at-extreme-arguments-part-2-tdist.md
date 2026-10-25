---
id: T-2474
title: "Scalar distributions at extreme arguments, part 2: TDIST of 1E+300, POISSON with a negative mean, NEGBINOMDIST and PERMUTATIONA limits (gen-m9, 44 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
TDIST/T.DIST.2T of x = 1E+300, POISSON/POISSON.DIST with a negative mean, NEGBINOMDIST/NEGBINOM.DIST at a negative
number_f or an underflowing probability, and PERMUTATIONA at a negative fraction or a huge number give Excel's results.

## Context
Split 2026-10-25 (gap triage, sonnet refill). The NEGBINOM number_f in (-1, 0) cells (d208bbafc3bc1997 A231, 3b7990bd4e3307ba A312, 1db18890916ee643 A124) moved to T-2700. The PERMUTATIONA truncation cells (Excel 1: 0620505280d2ea6e A280, 31f09e6c48ca245c A227, 5395b8a427764a3f A385, 5cebac1f02bf30f2 A148, a5342a7e24fd803e A370) moved to T-2701. This task keeps TDIST/T.DIST.2T of 1E+300, the POISSON negative-mean cells, NEGBINOMDIST's underflow and PERMUTATIONA's huge number. The counts below exclude the moved cells.

Gap triage of the T-2256 gen-m9 sweep, the sibling of T-2435 for the M-to-Z functions. statistics.md "Scalar distributions
at extreme arguments, part 2 (T-2256)" lists the rules: TDIST and T.DIST.2T of 1E+300/1E+307 are `#NUM!` (12 cells, 3 more
nested), POISSON with x truncating to 0 and a negative mean is EXP(-mean) (9 cells), NEGBINOMDIST rejects number_f in
(-1,0) and returns 0 where the probability underflows (8 cells), PERMUTATIONA truncates number_chosen toward 0 before the
sign check and rejects a number of about 1E+15 (10 cells). The cuts (TDIST between 1E+15 and 1E+300, PERMUTATIONA's number)
are not in these cells: implement the observed side of each and record the chosen cut as UNVERIFIED, or file a probe.
Code: functions/statistical/t_dist.rs (TDIST, T.DIST.2T), poisson.rs, binom.rs or discrete.rs (NEGBINOM), permut.rs.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 34 known-failure lines in eval/baseline/gen-m9.txt, 34 visibly off (42 before the T-2700 and T-2701 splits).
Examples:
- gen-m9/607e85cfee83a689 Calc!A189 `=POISSON(0,(-1),0)`: Excel 2.718281828459045, ours #NUM!
- gen-m9/775d053de498b8c7 Calc!A26 `=T.DIST.2T(1E+307,3)`: Excel #NUM!, ours 0.0
- gen-m9/928d08e3e1eecb5f Calc!A36 `=TDIST(1E+300,7,1)`: Excel #NUM!, ours 0.0
- gen-m9/bbab6562e88b70e8 Calc!A96 `=T.DIST.2T(1E+300,7)`: Excel #NUM!, ours 0.0
- gen-m9/d5fd9e05e9e6f466 Calc!A352 `=T.DIST.2T(1E+300,9)`: Excel #NUM!, ours 0.0
Cells:
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A353
- gen-m9/09b47629f54b9817.case.json (1): Calc!A34
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A184
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A159
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A30
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A246
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A295
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A64
- gen-m9/5919660b7bb4d195.case.json (2): Calc!A124, A374
- gen-m9/607e85cfee83a689.case.json (2): Calc!A189, A226
- gen-m9/775d053de498b8c7.case.json (2): Calc!A26, A152
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A311
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A189
- gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A36
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A81
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A33
- gen-m9/9e2225ead8a6a07b.case.json (1): Calc!A305
- gen-m9/b9a32ce9a1fd0d71.case.json (4): Calc!A31, A195, A221, A378
- gen-m9/bbab6562e88b70e8.case.json (1): Calc!A96
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A24
- gen-m9/c2684066d8df263e.case.json (1): Calc!A72
- gen-m9/c4635963fc926284.case.json (1): Calc!A300
- gen-m9/c7664e494e418395.case.json (1): Calc!A189
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A280
- gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A352
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A341
- gen-m9/eccd03df42e40006.case.json (1): Calc!A183
- gen-m9/eedd629f3d69fdda.case.json (1): Calc!A40

gen-m9 engineering and database sweep (gap triage 2026-10-23, T-2258): T.DIST of 1E+307 is `#NUM!` too: `BITRSHIFT(T.DIST(1E+307,9,"TRUE")/0,-2)` and `DPRODUCT(Db!B1:D9,T.DIST(1E+307,2,TRUE)-1,Db!F1:F2)` are `#NUM!` (ours T.DIST is 1, so `#DIV/0!` and a bad field `#VALUE!`).
Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas, 2 visibly off.
Examples:
- gen-m9/8fa8b572ebc1f001 Calc!A382 `=BITRSHIFT((T.DIST(1E+307,9,"TRUE")/0),-2)`: Excel #NUM!, ours #DIV/0!
- gen-m9/e707d43a891ba9b5 Calc!A391 `=DPRODUCT(Db!B1:D9,(T.DIST(1E+307,2,TRUE)-(2*0.5)),Db!F1:F2)`: Excel #NUM!, ours #VALUE!
Cells:
- gen-m9/8fa8b572ebc1f001.case.json (1): Calc!A382
- gen-m9/e707d43a891ba9b5.case.json (1): Calc!A391

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `T.DIST.RT(1E+300,7)` is `#NUM!` like TDIST (ours 0), and `POISSON(0,-0.5,Data!E$6)` (cumulative) is EXP(0.5) = 1.6487212707001282 (ours `#NUM!`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/fec39efaba9284c3 Calc!A37 `=FORECAST(Data!E8,Data!D4:E6,T.DIST.RT(1E+300,7))`: Excel #NUM!, ours #N/A; inner `T.DIST.RT(1E+300,7)` Excel #NUM!, ours 0.0
- gen-m9/165db58d1540f014 Calc!A105 `=CONFIDENCE((POISSON(0,(-0.5),Data!E$6)*CHITEST(Data!E1,(-2.5))),VAR(Data!D5:D11,Data!B8:B9),6)`: Excel #N/A, ours #NUM!; inner `POISSON(0,(-0.5),Data!E$6)` Excel 1.6487212707001282, ours #NUM!
Cells:
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A37
- gen-m9/165db58d1540f014.case.json (1): Calc!A105

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2258): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2256): 33 of the 34; b9a32ce9a1fd0d71 A195 (`NEGBINOMDIST(-0.5,..)`, number_f in (-1, 0)) is T-2700's rule and stays with it: these 34 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests for each rule at the cited arguments.
- [x] statistics.md's T-2256 entry gets an "Ours:" line naming this task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Last-digit accuracy of these functions (T-0969, T-2055, T-2146).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): every cell it owns has the wrong type, error or text (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.

## Outcome
- T.DIST (cumulative), T.DIST.RT, T.DIST.2T and TDIST are `#NUM!` where `x²` overflows (cut UNVERIFIED).
- POISSON/POISSON.DIST with a negative mean are EXP(-mean) when x truncates to 0, `#NUM!` otherwise; this also closes
  T-2628 (probes/poisson-negative-mean A1-A7, gen-m9/23d82b62e9a85bdd A208).
- NEGBINOM at an underflowing probability is 0: the binomial walk (discrete.rs) renormalized only past 2^600, which
  assumes a ratio p/q below 2^53; a larger ratio now renormalizes once a term passes 1, and a step that still
  overflows falls back instead of returning NaN.
- PERMUTATIONA takes PERMUT's 2^31 - 1 number limit (cut UNVERIFIED).
Eval delta: gen-m9 known failures 4216 -> 4179 (fixed 37), probes 2686 -> 2665 (fixed 21: poisson-negative-mean and
the t2436 sub-expression cells), 0 regressions; visible parity 4430 (lock 4519). b9a32ce9a1fd0d71 A195 is T-2700's.
