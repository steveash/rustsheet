---
id: T-2146
title: "BINOM.DIST, BINOM.DIST.RANGE and NEGBINOM.DIST last digits: fit Excel's evaluation (70 ironcalc-calc BINOM.xlsx known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2145, T-2510]
---

## Goal
BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, NEGBINOM.DIST and NEGBINOMDIST give Excel's doubles. T-0927's residue in
`ironcalc-calc/STATISTICAL/BINOM.xlsx` has no open owner today: T-2055 lists the binomial family as out of scope
and points back to T-0927, which is done.

## Context
- The oracle sweep (T-2145) is landed: probes-numeric/t2145-binomial-sweep, t2145-negbinom-sweep
  and t2145-binom-range-sweep (statistics.md "Discrete distributions"). T-0927 tried and ruled out the exact sum, Loader's saddle-point pmf (R's
  `dbinom_raw`: 49 of 122) and `exp(a*ln x)` times the exact remaining factor, all on the cached cells alone.
- Known failures (eval/baseline/ironcalc-calc.txt, gap triage 2026-10-20): 70 lines, BINOM.xlsx sheet by sheet:
  - BINOM.DIST: 18 (15 NumericUlp, for example E3: Excel 0.23925373020405763, ours ...576; 3 NumericClose:
    E24, E32, E67, for example E24 `BINOM.DIST(2,10,0.999999,TRUE)`: Excel 4.499992001038836E-47, ours ...802E-47).
  - BINOM.DIST.RANGE: 17 (14 NumericUlp, for example E2: 0.2552983299999998 against ...999; 3 NumericClose: E10,
    E21, E23).
  - NEGBINOM.DIST: 35 (28 NumericUlp, for example E4: 0.02800000000000001 against 0.028000000000000004; 7
    NumericClose: E3, E26, E30, E37, E40, E74, E75).
  - Ungated: probes-numeric/m9-legacy-binomdist (21 failing, for example A5 `BINOMDIST(3,10,0.5,TRUE)`:
    0.17187500000000006 against the exact 0.171875) and t1048-negbinomdist (12 failing, for example A7
    `NEGBINOMDIST(3,3,0.75)`: 0.06591796875000003 against the exact 0.06591796875).
- Evidence and hypotheses (statistics.md "Discrete distributions: within an ulp of the exact sum"):
  `NEGBINOM.DIST(0,100,0.01,TRUE)` is `exp(100*ln(0.01))` in doubles bit for bit, Excel's BETA.DIST prefactor
  ("Incomplete beta: Excel's closed forms and the best candidate", T-1744). So a likely shape for the cdf is the
  regularized incomplete beta, `BINOM.DIST(k,n,p,TRUE) = I_{1-p}(n-k, k+1)`, with Excel's own prefactor. The pmf may be
  `exp` of a log-gamma sum, or a ratio. Try them in the fit lab (`cd tools && uv run rustsheet-fitlab`,
  docs/architecture/fit-lab.md). If the cdf is the incomplete beta, share `statistical/incomplete_beta.rs` (T-0970;
  T-2119 moves BETA.DIST onto it), not a second kernel.
- Code: `crates/rustsheet-engine/src/functions/statistical/discrete.rs` (T-0927's exact walk, capped at 2^10 steps
  with statrs past it).

Visibly-off cells (gap triage 2026-10-21, decision 0083): 14 cells, 14 numeric past half a unit in the 15th digit and 0 with a wrong type, error or text, blamed on NEGBINOM.DIST 6, BINOM.DIST 4, BINOMDIST 2, NEGBINOMDIST 2.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| ironcalc-calc/STATISTICAL/BINOM.xlsx BINOM.DIST!E24 | `=_xlfn.BINOM.DIST(A24,B24,C24,D24)` | 4.499992001038836e-47 | 4.499992001038802e-47 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx BINOM.DIST!E32 | `=_xlfn.BINOM.DIST(A32,B32,C32,D32)` | 9.90100000000073e-197 | 9.90100000000087e-197 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E3 | `=_xlfn.NEGBINOM.DIST(A3,B3,C3,D3)` | 6.50284892159999e-10 | 6.502848921600006e-10 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E30 | `=_xlfn.NEGBINOM.DIST(A30,B30,C30,D30)` | 6.599988000005536e-59 | 6.599988000005497e-59 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E37 | `=_xlfn.NEGBINOM.DIST(A37,B37,C37,D37)` | 1.0000000000000348e-200 | 1.000000000000002e-200 |
| ironcalc-calc/STATISTICAL/BINOM.xlsx NEGBINOM.DIST!E40 | `=_xlfn.NEGBINOM.DIST(A40,B40,C40,D40)` | 1.690659363904351e-142 | 1.6906593639043344e-142 |
| probes-numeric/m9-legacy-binomdist Calc!A33 | `=BINOMDIST(2,5,0.999,TRUE)` | 9.985006000000038e-09 | 9.985006000000026e-09 |
| probes-numeric/m9-legacy-binomdist Calc!A34 | `=BINOM.DIST(2,5,0.999,TRUE)` | 9.985006000000038e-09 | 9.985006000000026e-09 |
| probes-numeric/m9-legacy-binomdist Calc!A35 | `=BINOMDIST(2,5,0.999,FALSE)` | 9.980010000000044e-09 | 9.980010000000027e-09 |
| probes-numeric/m9-legacy-binomdist Calc!A36 | `=BINOM.DIST(2,5,0.999,FALSE)` | 9.980010000000044e-09 | 9.980010000000027e-09 |
| probes-numeric/t1048-negbinomdist Calc!A4 | `=NEGBINOMDIST(10,2,0.9)` | 8.909999999999952e-10 | 8.909999999999981e-10 |
| probes-numeric/t1048-negbinomdist Calc!A5 | `=NEGBINOMDIST(1,10,0.1)` | 8.999999999999998e-10 | 9.000000000000005e-10 |
| probes-numeric/t1048-negbinomdist Calc!B4 | `=NEGBINOM.DIST(10,2,0.9,FALSE)` | 8.909999999999952e-10 | 8.909999999999981e-10 |
| probes-numeric/t1048-negbinomdist Calc!B5 | `=NEGBINOM.DIST(1,10,0.1,FALSE)` | 8.999999999999998e-10 | 9.000000000000005e-10 |

gen-m9 sweep (gap triage 2026-10-23, T-2255). BINOM.DIST, BINOMDIST and BINOM.DIST.RANGE values visibly off (8 cells, tails near
1E-100).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 8 known-failure lines in eval/baseline/gen-m9.txt, 8 formulas.
Examples:
- gen-m9/0a614cfe974b59c3 Calc!A170 `=BINOM.DIST(Data!D$10,1,1E-100,FALSE)`: Excel 9.99999999999989e-101, ours 1e-100
Cells:
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A170
- gen-m9/27601819397dd56e.case.json (1): Calc!A280
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A174
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A276
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A360
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A380
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A397
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A113

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: BINOMDIST, BINOM.DIST and BINOM.DIST.RANGE last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 21 known-failure lines in eval/baseline/gen-m9.txt, 21 formulas.
Examples:
- gen-m9/146b46e74f2dac3b Calc!A346 `=BINOMDIST(3,10,0.05,ISLOGICAL(Data!B$8))`: Excel 0.010475059441406248, ours 0.010475059441406252
Cells:
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A346
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A164
- gen-m9/4250f880ab640c49.case.json (1): Calc!A88
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A382
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A167
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A357
- gen-m9/623663cacea53bbc.case.json (1): Calc!A199
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A38
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A303
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A41
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A323
- gen-m9/a013a45a8fbcab4f.case.json (3): Calc!A145, A250, A366
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A366
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A37
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A52
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A45
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A289
- gen-m9/d8266bc15533702a.case.json (1): Calc!A287
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A32

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): NEGBINOMDIST and NEGBINOM.DIST values off Excel's, mostly at tiny or near-1 probabilities where Excel's formulation carries error the exact value does not (`NEGBINOMDIST(5,3,1E-20)` Excel 2.1000000000000084E-59, exact 2.1E-59; `NEGBINOMDIST(0,1,1E-200)` 9.99999999999978E-201). Domain edges are T-2474's.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 85 known-failure lines in eval/baseline/gen-m9.txt, 40 visibly off.
Examples:
- gen-m9/329b177b5d5165ad Calc!A329 `=NEGBINOMDIST((Data!D11/AVERAGEA(Data!E1:E5,Data!B9:B9,Data!D9:D11)),6,1e-15)`: Excel 1.0000000000000072e-90, ours 1.0000000000000004e-90
- gen-m9/44a8841352dc48e9 Calc!A157 `=STANDARDIZE((IFNA(0.1,Data!$D12)/YEAR(1)),Data!B8,NEGBINOMDIST(7,9,0.025))`: Excel -2042715136323.223, ours -2042715136323.2314
- gen-m9/583ca2baea0bb30a Calc!A123 `=NEGBINOM.DIST(7,2.5,1-1E-10,(ISNUMBER(Data!D10)=(Data!$B7<>1000000000000000.0)))`: Excel 8.000004631861894e-70, ours 8.000004631861925e-70
- gen-m9/787ba8d801105a60 Calc!A71 `=NEGBINOMDIST(2,3,0.05)`: Excel 0.0006768749999999995, ours 0.0006768750000000001
- gen-m9/c2684066d8df263e Calc!A1 `=NEGBINOM.DIST(4,3,1E-20,0)`: Excel 1.4999999999999724e-59, ours 1.4999999999999998e-59
Cells:
- gen-m9/097af1ca33638670.case.json (1): Calc!A177
- gen-m9/09b47629f54b9817.case.json (1): Calc!A322
- gen-m9/0bbab6206b5c9a07.case.json (2): Calc!A25, A87
- gen-m9/11eb3ab4f4f8104c.case.json (3): Calc!A188, A312, A374
- gen-m9/139684e68fb2fc3b.case.json (2): Calc!A228, A380
- gen-m9/17c110a53924beb8.case.json (1): Calc!A11
- gen-m9/1b31092a0d79e5f1.case.json (1): Calc!A197
- gen-m9/1db18890916ee643.case.json (1): Calc!A86
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A45
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A329
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A325
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A250
- gen-m9/3fb9cecf2c23169f.case.json (1): Calc!A100
- gen-m9/43c6227e0ff368c7.case.json (2): Calc!A203, A327
- gen-m9/44a8841352dc48e9.case.json (3): Calc!A157, A325, A363
- gen-m9/46ac3ef5e61583ca.case.json (2): Calc!A305, A347
- gen-m9/583ca2baea0bb30a.case.json (2): Calc!A30, A123
- gen-m9/5919660b7bb4d195.case.json (2): Calc!A10, A276
- gen-m9/5cebac1f02bf30f2.case.json (2): Calc!A241, A317
- gen-m9/607e85cfee83a689.case.json (5): Calc!A22, A196, A258, A270, A394
- gen-m9/6e18550e9fc9c1b9.case.json (2): Calc!A5, A160
- gen-m9/775d053de498b8c7.case.json (2): Calc!A204, A390
- gen-m9/787ba8d801105a60.case.json (1): Calc!A71
- gen-m9/7980ffb6095a340d.case.json (2): Calc!A181, A257
- gen-m9/7f7ca726c3f68554.case.json (1): Calc!A124
- gen-m9/7fd0a98183c6babf.case.json (2): Calc!A37, A379
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A155
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A183
- gen-m9/941c409532603d32.case.json (2): Calc!A25, A273
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A368
- gen-m9/9bc247733a1ca9fa.case.json (1): Calc!A340
- gen-m9/9e2225ead8a6a07b.case.json (3): Calc!A238, A275, A362
- gen-m9/a5342a7e24fd803e.case.json (3): Calc!A71, A109, A375
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A259
- gen-m9/afc00c8b70e78baf.case.json (2): Calc!A43, A353
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A43
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A48
- gen-m9/c2684066d8df263e.case.json (4): Calc!A1, A149, A273, A335
- gen-m9/c4635963fc926284.case.json (1): Calc!A148
- gen-m9/c7664e494e418395.case.json (3): Calc!A110, A262, A300
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A143
- gen-m9/d208bbafc3bc1997.case.json (2): Calc!A117, A307
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A27
- gen-m9/d5fd9e05e9e6f466.case.json (2): Calc!A49, A202
- gen-m9/d8f44d556674c2fd.case.json (3): Calc!A167, A281, A395
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A15
- gen-m9/eccd03df42e40006.case.json (1): Calc!A169
- gen-m9/ede0323c389fc703.case.json (3): Calc!A173, A287, A363
- gen-m9/f82473275380882e.case.json (1): Calc!A392

Sequencing (T-0098, 2026-10-28): waits on T-2510, which ports Excel's bgrat for BINOM and NEGBINOM; fit the last
digits on the ported kernel, not the current one, so the work is not done twice.

### Cases owned (T-2981)
Routed by T-2981 (eval of 2026-10-28 at `main` 9586958b): the BINOM.DIST and NEGBINOM.DIST pmf cells (interior k; dbinom_raw on T-2507's saddle point); the cases' BETA.DIST density cells are T-2507's. 4,415 visibly-off cells in 4 cases, per case:
- probes-numeric/t2513-r3-binom-pmf-a: BINOM.DIST pmf (interior) 1,213
- probes-numeric/t2513-r3-binom-pmf-hold: BINOM.DIST pmf (interior) 628, NEGBINOM.DIST pmf 780
- probes-numeric/t2513-r3-binom-pmf-n: BINOM.DIST pmf (interior) 997, NEGBINOM.DIST pmf 629
- probes-numeric/t2513-verify-r3-binom-pmf-h: BINOM.DIST pmf (interior) 54, NEGBINOM.DIST pmf 114

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 93 known-failure lines (8 from T-2255, 85 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Visible parity: the 14 visibly-off cells listed under Context (2026-10-21) have Excel's value to 15 significant digits, or its type, error and text: the 6 lines in eval/baseline/ironcalc-calc.txt disappear (`make eval-baseline`); the 8 cells of the ungated probes-numeric suite pass in `make eval`, with no new known-failure lines in any baseline.
- [ ] statistics.md "Discrete distributions" records the fit: each candidate's count on BINOM.xlsx's 148 numeric
      cells, on T-2145's sweep and on the two probes-numeric cases, including the candidates that lost.
- [ ] The formulation that wins is implemented. The BINOM.xlsx known-failure lines it fixes disappear from
      eval/baseline/ironcalc-calc.txt, with no new ones in any gated suite. The target is all 70 lines; say which
      ones remain and why.
- [ ] No BINOM.INV/CRITBINOM cell regresses (they search on the same cdf).
- [ ] `make check` green; reviewers approve.

## Out of scope
- POISSON.DIST (T-2055), HYPGEOM.DIST (T-0977, T-1896), BINOM.INV with probability_s 0 (T-2050).

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (binomial and negative-binomial values that differ in the 14th-15th digit), so this task is in the campaign. Its target is that every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0), or has Excel's type, error and text; the visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet when that target is reached move to a follow-up task filed without the key and named in the Outcome (decision 0083). The fit needs T-2145's oracle sweep first.

## Phase 1 triage (T-0090, 2026-10-27)
It owns 71 of BINOM.DIST.RANGE's 73 needed cells, 534 BINOM.DIST and 121 NEGBINOM.DIST cells: despite the "last digits" title these cells are visibly off. Priority raised so it leads its stage in M9.
