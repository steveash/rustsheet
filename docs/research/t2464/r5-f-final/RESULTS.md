# T-2586 round 5 — r5-f-final (integrator): one frozen model of F.DIST(x, d1, d2, FALSE)

**Result: not cracked.** The integrated model is not bit-exact and does not reach visible parity. It is the best
model so far: on all 64,922 F-density cells in probes-numeric plus cells.tsv, 47,113 (72.6%) are within 15 digits,
against 46,994 for round 4's M-sd with r5-f-edges' rules and 32,358 for the correctly rounded density. On the new
untouched hold-out it gets **126 / 522 / 722 of 1,584** value cells (bit-exact / within 4 ulp / within 15 digits),
against M-sd+rules 123/525/721 and CR 63/269/444. All 35 rule cells (errors and zeros) in that set are right, and
45 of the 50 literal edge cells are within 15 digits.

What still blocks parity is one thing: the size of the power term P = c·ln x. When |P| < 4, 99-100% of cells are
within 15 digits. When |P| >= 32, 10-36% are. The error spread grows as 1.3-2 times ulp(|P|)/sqrt(12). That is the
signature of at least one stored double rounding at P's scale that the model does not have. It matches r5-f-edges'
E = fl(Q + fl(c·LN x)) + R, with Q unidentified.

All counts below are bit-exact / within 4 ulp / within 15 digits (half a unit of the 15th significant digit of
Excel's value). Errors and zeros must match exactly (r5-f-edges `elib.grade`).

## 1. The model (frozen before the hold-out was read)

`fdensity_excel.py` (sha256 `e1b368a2…5502d1173`, written 16:35 after the hold-out fill and before `final_score.py
hold` first read the sidecar at 16:36). Call it as `fdensity_excel.value(x, d1, d2, fin="div")`. It returns a float or an
error string. `fin="mulr"` uses fl(Y·fl(1/x)) as the last step. Each piece is imported, and the module fixes only
their order:

| piece | rule | source (verdict) |
|---|---|---|
| rules | df truncated; trunc df < 1 or > 1E10 → #NUM!; x < 0 → #NUM!; x = 0 → #NUM!/1/0 for trunc(d1) = 1/2/>=3; 0 < x < 2^-1022 → #DIV/0!; fl(m·x) overflow → #NUM!; E < -800 → 0; subnormal Y kept; F < 2^-1022 → 0 | r5-f-edges `edgemodel` (confirmed) |
| side, r | mx = fl(m·x); side 1 (mx <= n): r = fl(mx/n), c = a; side 2: r = fl(n/mx), c = -b | r4 (verified) |
| P | fl(c · LN(x)), stored | r4 |
| H | -h · L(r), extended; L = Excel's log1p (14-term series to 0.375, LN(fl(1+r)) above) | r4. r5-f-smallr's xs1/dd were refuted |
| K | K_exact + delta·2^-53; delta = M-sd's side bracket - 0.42 when min(m,n) >= 16, else M-sd's delta | r5-f-bigconst `bcmodel.K_of` (confirmed for bit-exact counts) |
| final | E = K + P + H (extended); Y = fl(EXP_V6(E)); F = fl(Y/x) | r4 M-sd |

The following are not used: r5-f-combine's stored-sum order hkp-sd and its hybrid (both refuted), and r5-f-smallr's
xs1, xs and dd (refuted by their verifier).

Sanity check before freezing (on in-sample data only): the module equals `bcmodel.value` on 600 of 600 random
`ba` cells, for both `div` and `mulr`. It equals `edgemodel.value` on 600 of 600 random r3 `a` cells with
min(d) < 16.

## 2. The hold-out t2586-r5-f-final-hold (built and filled first, read once)

`mkhold.py` builds it with seed 25865 and 3,218 cells:
- **Shapes:** 40 random shapes with d1, d2 log-uniform in [1, 500], about 35% of them written as non-integer
  literals, plus 4 shapes with one df in {1, 1.7, 2, 3, 3.6, 4}. Any shape that an earlier F.DIST(.,FALSE) case
  uses (after truncation) is excluded.
- **Runs:** for each shape, 4 runs of 8 consecutive doubles, one per region: side 1 r <= 0.375, side 1 r > 0.375,
  side 2 r > 0.375, side 2 r <= 0.375. The small r is drawn log-uniform in [1e-3, 0.375].
- **Singletons:** 4 random x per shape, x = e^U(-6,6)·n/m.
- **Edge cells:** 50 literal-argument cells.

The first fill failed: Graph returned "unsupported workbook" because of the subnormal literals 1E-310 and 4E-320.
Those two cells were replaced by EXP(-715) and EXP(-736), the case was rewritten before any value existed, and the
second fill succeeded (excel-graph, 16:34). All 1,584 x echoes match.

### Hold-out value cells (n = 1,584)

| region | n | M-final | M-final-mulr | M-sd+rules | M-sdm+rules | CR+rules |
|---|---|---|---|---|---|---|
| all | 1584 | **126/522/722** | 140/523/729 | 123/525/721 | 135/520/724 | 63/269/444 |
| side 1, r <= 0.375 | 395 | 22/89/140 | 24/90/140 | 22/89/140 | 24/89/141 | 9/60/87 |
| side 1, r > 0.375 | 364 | 15/117/178 | 16/117/178 | 12/118/176 | 17/117/176 | 3/31/94 |
| side 2, r > 0.375 | 373 | 22/113/163 | 25/114/168 | 17/115/164 | 18/114/165 | 1/35/78 |
| side 2, r <= 0.375 | 417 | 32/168/206 | 40/167/208 | 37/168/206 | 41/165/207 | 15/108/150 |
| small df (min < 5) | 251 | 25/57/81 | 25/57/83 | 25/57/81 | 25/57/83 | 5/51/58 |
| mid (5..15) | 615 | 25/244/322 | 35/244/323 | 25/244/322 | 35/244/323 | 13/135/231 |
| large (>= 16) | 683 | 41/186/284 | 45/187/288 | 38/189/283 | 40/184/283 | 10/48/120 |
| edge x (x < 1e-6, x > 1e6 or F < 1e-280) | 8 | 0/0/0 | 0/0/0 | 0/0/0 | 0/0/0 | 0/0/0 |
| rule (error / 0) | 35 | 35/35/35 | 35/35/35 | 35/35/35 | 35/35/35 | 35/35/35 |

This hold-out is much harder than the workers' hold-outs (46% within 15 digits, against about 80% on theirs). It
spreads x over e^±6·n/m and r down to 1e-3, so most cells have |P| >= 32 (section 4).

### Hold-out edge cells (n = 50)

M-final gets 35/45/45; CR+rules gets 27/40/43. All 23 rule cells are right:
- x = 0 at d1 = 1, 1.9, 2, 2.99, 3.5 and 250;
- x < 0;
- df 0.9, 0.99 and 0.5;
- 10000000000.5 truncated (a value), and 10000000001 (#NUM!);
- EXP(-715) and EXP(-736) (#DIV/0!);
- the 1E300·1e9 overflow and 1.7E306·200 (#NUM!);
- underflows to 0.

The coercion cells TRUE → 1 and "2" → 2 are within 15 digits. So are the 15-digit truncation of
0.123456789012345678 and 1.00000000000000001, and non-integer df such as (123.456, 321.987).

The 5 visibly-off cells all have a large |P|:
- F.DIST(1E-300,1,1) = 3.18309886183795**23**e149, model …**46**e149;
- F.DIST(1E-300,2,9) = 0.99999999999997**86**, model 1.00000000000002**38**;
- F.DIST(1E-150,1,300);
- F.DIST(35,3,60);
- F.DIST(0.01,60,3).

## 3. Every earlier F set, scored once with the frozen model (`final_sets.out`)

| set | n | M-final | M-final-mulr | M-sd+rules | M-sdm+rules | CR+rules |
|---|---|---|---|---|---|---|
| cells.tsv F pdf | 1599 | 473/1365/1462 | 521/1364/1461 | 474/1363/1462 | 523/1363/1461 | 152/1077/1308 |
| t2220 sweep d1-1 | 300 | 70/238/261 | 73/236/260 | 70/238/261 | 73/236/260 | 14/179/228 |
| t2220 sweep d2-1 | 270 | 52/208/237 | 61/210/237 | 52/208/237 | 61/210/237 | 22/170/208 |
| t2220 sweep pdf | 1000 | 335/892/936 | 372/891/936 | 337/890/936 | 375/890/936 | 111/703/845 |
| r3-fdensity a | 3168 | 796/2634/2946 | 874/2640/2944 | 822/2638/2941 | 897/2640/2941 | 337/1958/2638 |
| r3-fdensity hold | 1440 | 337/1125/1252 | 374/1129/1251 | 337/1119/1255 | 374/1123/1253 | 114/733/1060 |
| r3-fdensity holdsw | 400 | 145/400/400 | 192/400/400 | 145/400/400 | 192/400/400 | 37/311/395 |
| r3-fdensity sw | 2400 | 1073/2384/2400 | 1116/2383/2400 | 1073/2384/2400 | 1116/2383/2400 | 585/2167/2227 |
| verify-r3-fdensity hold | 960 | 157/657/786 | 196/648/785 | 159/651/784 | 192/650/781 | 47/337/568 |
| r4-f-brcomp big | 2304 | 364/1335/1722 | 409/1333/1721 | 156/1302/1700 | 153/1304/1698 | 45/344/857 |
| r4-f-brcomp hold | 1536 | 308/1064/1199 | 348/1068/1196 | 287/1048/1198 | 309/1048/1199 | 111/646/904 |
| r4-f-power a | 5376 | 192/1600/2551 | 226/1610/2564 | 198/1607/2546 | 215/1618/2550 | 62/610/1336 |
| r4-f-power hold | 1536 | 33/348/746 | 32/351/745 | 29/355/733 | 32/352/730 | 14/117/294 |
| r4-f-small-df a | 6016 | 1695/4623/5060 | 1865/4624/5058 | 1695/4623/5060 | 1865/4624/5058 | 624/3575/4519 |
| r4-f-small-df hold | 1344 | 314/919/1029 | 335/918/1030 | 314/919/1029 | 335/918/1030 | 104/688/882 |
| r5-f-bigconst a | 6128 | 1390/4004/4495 | 1470/4008/4498 | 826/3979/4502 | 897/3988/4506 | 181/1150/2378 |
| r5-f-bigconst hold | 2448 | 544/1643/1847 | 580/1642/1846 | 350/1629/1841 | 389/1631/1835 | 67/432/1049 |
| r5-f-combine a | 6400 | 1265/4550/5053 | 1395/4543/5059 | 1281/4472/5015 | 1388/4472/5016 | 258/2014/2950 |
| r5-f-combine hold | 1536 | 372/1143/1292 | 475/1142/1289 | 287/1139/1280 | 358/1135/1275 | 41/412/604 |
| r5-f-smallr a | 7168 | 378/2410/3676 | 378/2412/3678 | 512/2443/3663 | 587/2438/3663 | 85/802/2006 |
| r5-f-smallr hold | 1536 | 54/510/866 | 64/510/860 | 65/489/851 | 65/484/846 | 20/226/459 |
| r5-f-edges a (own loader) | 1437 | 739/959/1033 | 764/954/1029 | 737/957/1035 | 762/953/1032 | 579/847/914 |
| r5-f-edges hold | 1396 | 459/960/1094 | 457/961/1093 | 459/960/1094 | 457/961/1093 | 312/827/990 |
| verify-r4-f-brcomp hold | 960 | 171/649/796 | 192/651/796 | 150/656/795 | 171/655/793 | 39/319/479 |
| verify-r4-f-power hold | 1000 | 48/306/517 | 52/312/515 | 43/313/506 | 52/314/505 | 4/72/202 |
| verify-r4-f-small-df hold | 896 | 312/852/886 | 355/854/884 | 312/852/886 | 355/854/884 | 88/644/803 |
| verify-r5-f-bigconst hold | 864 | 141/475/525 | 143/476/522 | 87/472/528 | 96/472/528 | 34/128/186 |
| verify-r5-f-combine hold | 960 | 84/569/698 | 96/570/691 | 75/562/700 | 84/561/692 | 13/145/270 |
| verify-r5-f-smallr hold | 960 | 40/285/626 | 38/282/624 | 26/289/635 | 33/289/633 | 14/136/355 |
| **all earlier sets** | 63338 | **12341/39107/46391** | 13453/39122/46372 | 11358/38957/46273 | 12406/38966/46235 | 4114/21769/31914 |
| **all + final hold** | 64922 | **12467/39629/47113** | 13593/39645/47101 | 11481/39482/46994 | 12541/39486/46959 | 4177/22038/32358 |

A note on the edges loader: the r5-f-edges cases encode x as `(H*2^26+L)*2^E`, which verify-r3-fdensity's
parser does not read. They are therefore read through r5-f-edges' own `s4_probe` loader, which checks echoes and
reads the 2^600-scaled column. The t2220 cdf and rt sets hold no density cells.

### By region, all earlier sets (n = 63,338)

| region | n | M-final | M-sd+rules | CR+rules |
|---|---|---|---|---|
| small df, r <= 0.375 | 11058 | 3676/9275/9724 | 3676/9275/9724 | 1498/7855/8959 |
| small df, r > 0.375 | 6098 | 1768/4752/5271 | 1768/4752/5271 | 591/3637/4714 |
| mid df, r <= 0.375 | 2806 | 641/2397/2609 | 641/2397/2609 | 347/1753/2355 |
| mid df, r > 0.375 | 4989 | 1579/4728/4867 | 1579/4728/4867 | 366/2823/3716 |
| large df, r <= 0.375 | 20277 | 1781/7994/11712 | 1550/7994/11666 | 353/3070/7145 |
| large df, r > 0.375 | 17373 | 2159/9224/11471 | 1407/9074/11399 | 295/1967/4361 |
| edge x | 616 | 8/52/107 | 8/52/107 | 3/35/81 |
| rule (error / 0) | 737 | 737/737/737 | 737/737/737 | 664/664/664 |

M-bc's constant improves only large df, and it improves it on both r branches. Small and mid df are unchanged,
because there M-final is M-sd.

## 4. Where the visibly-off cells are (`final_off.out`, `s1_offmap.out`, `s2_errscale.out`)

17,809 of 64,922 cells are not within 15 digits. By (side, r class, df class):

| class | off cells | shapes |
|---|---|---|
| side 2, r <= 0.375, large | 5634 | 109 |
| side 1, r <= 0.375, large | 3178 | 78 |
| side 1, r > 0.375, large | 3063 | 437 |
| side 2, r > 0.375, large | 2991 | 436 |
| small (all four) | 2331 | — |
| mid (all four) | 612 | — |

By |P| = |c·ln x|, all earlier sets:

| \|P\| | within 15 digits | rms error (units) | ulp(\|P\|)/√12 (units) |
|---|---|---|---|
| < 0.5 | 9019 of 9022 (100%) | 3.1 | 0.03 |
| [0.5, 4) | 14267 of 14392 (99%) | 3.9 | 0.74 |
| [4, 32) | 15684 of 18016 (87%) | 11.7 | 5.9 |
| [32, 256) | 6297 of 17267 (36%) | 56.9 | 42 |
| [256, 2048) | 387 of 3895 (10%) | 402 | 201 |

With |P| < 4, H and K are good enough for parity at every H size, including |H| up to 2048. There are 159
within-15 failures there in all: 124 in earlier sets and 35 in the final hold-out. Of these, 154 are at small df
(min(d) < 5), 4 are at large df and 1 is at mid df. The 35 in the hold-out are almost all shapes with one df of 1
or 2 and a large partner: (160,1), (1,156), (321,2), (472,1). That is r4's known small-df constant residue (the
stirlerr e-table at e(0.5) and e(1) against a large h), and the rms there is 3-4 units. At |P| >= 32 the error tracks the ulp of a double of P's size, at 1.35-2 times the spread of one
rounding. So at least one stored double at |P| scale is missing from the model, or misplaced: r5-f-edges' Q in
E = fl(Q + fl(c·LN x)) + R, or a split of P such as r5-f-smallr's fl(h·LN x) - fl(o·LN x). The final hold-out
shows the same pattern: P < 0.5 100%, [4, 32) 71%, [32, 256) 33%, [256, 2048) 4%. The hold-out has a few classes
at small P and large H (H >= 32) that score worse than on earlier sets (11 of 42). They are those same df = 1 / 2 shapes with a large partner (so h, and therefore H, is large).

Below |P| = 4 the model is effectively at parity. The ~36% of all probe cells with |P| >= 32 need Excel's exact
stored-sum structure, because no constant can be fitted there: the spread is at P's own quantum.

## 5. Hypotheses scored (all frozen before the hold-out was read)

| model | final hold-out | all earlier sets | verdict |
|---|---|---|---|
| M-final (rules + M-bc K + M-sd chain, div) | 126/522/722 | 12341/39107/46391 | **adopted**: best within 15 digits overall, best bit-exact among the div models |
| M-final-mulr | 140/523/729 | 13453/39122/46372 | most bit-exact (+1,126 cells overall); 19 fewer within 15 digits on earlier sets but 7 more on the hold-out; a coin-flip, not adopted |
| M-sd + rules (r4 baseline) | 123/525/721 | 11358/38957/46273 | baseline |
| M-sdm + rules | 135/520/724 | 12406/38966/46235 | baseline |
| CR + rules | 63/269/444 | 4114/21769/31914 | baseline |

Not re-scored here (refuted by their verifiers): hkp-sd, hyb (r5-f-combine), and xs1, xs, dd (r5-f-smallr).

## 6. For the integrator / engine port

- **Model:** `docs/research/t2464/r5-f-final/fdensity_excel.py`, `value(x, d1, d2, fin="div")`. `rule()` holds the
  argument rules and `exponent()` builds E. Each line maps to one Excel step. The constant needs
  `bcmodel.K_of(m, n, side)`, which is the exact -lnB ± a·ln(m/n) plus a delta table: r3's stirlerr e-table and
  M-sd's bracket. An engine has to carry that delta.
- **Accuracy:** it reaches visible parity only for |c·ln x| < 4 (99-100%; the misses there are shapes with one df
  of 1 or 2 and a large partner df), and is about 87% for |P| in [4, 32).
  It is still far off for |P| >= 32. Do not claim parity there.

## Files

- `mkhold.py`: hold-out generator, and the plan used to read the hold-out back.
- `fdensity_excel.py`: the frozen model.
- `final_score.py`: `hold`, `sets` and `off` reports. Outputs are `final_hold.out`, `final_sets.out` and
  `final_off.out`.
- `s1_offmap.py`, `s2_errscale.py`: diagnostics run after the scores (`.out` beside each).
- Probe: `eval/suites/probes-numeric/cases/t2586-r5-f-final-hold.case.json`, 3,218 cells, filled.
- `exact_cache.pkl`: git-ignored.
