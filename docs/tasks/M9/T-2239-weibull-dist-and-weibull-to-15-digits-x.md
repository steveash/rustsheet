---
id: T-2239
title: "WEIBULL.DIST and WEIBULL to 15 digits: (x/lambda)^k through exp(k*ln(x/lambda)), as Excel's tail values show (18 visibly-off sweep cells)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
WEIBULL.DIST and WEIBULL agree with Excel to 15 significant digits on the in-domain cells (x > 0)
of the three Weibull sweeps. Today 18 of them are visibly off. Excel's tail densities are less
accurate than the true value, so this means matching Excel's evaluation, not computing a more
accurate value.

## Context
Gap triage 2026-10-21 (`make eval` on `main` d77d01b0). `functions/statistical/weibull.rs`
`weibull_dist_of` calls `statrs::distribution::Weibull` (statrs 0.18: `cdf` = `-expm1(-x.powf(k) * l.powf(-k))`,
`pdf` = `k/l (x/l)^(k-1) exp(-x^k l^-k)`). T-0978 owns the last digits: it targets the m9 sweep and the two ironcalc
cells bit for bit but has no 15-digit (visible-parity) stage, so the visibly-off sweep cells had no
visible-parity task (decision 0083).

The visibly-off cells (excel-graph; probes-numeric is ungated, so read them from `make eval` /
`make eval-show CASE=probes-numeric/<case>.case.json`):

| case / cell | formula | Excel | ours |
|---|---|---|---|
| m9-weibull-dist A22 | `WEIBULL.DIST(1000,0.5,1,FALSE)` | 2.9199312344877555e-16 | 2.9199312344877447e-16 |
| m9-weibull-dist A58 | `WEIBULL.DIST(5,1,1,FALSE)` | 0.006737946999085473 | 0.006737946999085467 |
| m9-weibull-dist A60 | `WEIBULL.DIST(10,1,1,FALSE)` | 4.539992976248477e-05 | 4.5399929762484854e-05 |
| m9-weibull-dist A62 | `WEIBULL.DIST(30,1,1,FALSE)` | 9.357622968840141e-14 | 9.357622968840175e-14 |
| m9-weibull-dist A64 | `WEIBULL.DIST(100,1,1,FALSE)` | 3.7200759760206773e-44 | 3.720075976020836e-44 |
| m9-weibull-dist A88 | `WEIBULL.DIST(1000,1,9,FALSE)` | 6.177532755504216e-50 | 6.177532755504306e-50 |
| m9-weibull-dist A104 | `WEIBULL.DIST(10,2,1,FALSE)` | 7.440151952041357e-43 | 7.440151952041672e-43 |
| m9-weibull-dist A124 | `WEIBULL.DIST(5,2,9,FALSE)` | 0.09067205826292972 | 0.09067205826292979 |
| m9-weibull-dist A130 | `WEIBULL.DIST(100,2,9,FALSE)` | 5.969559326940941e-54 | 5.969559326940687e-54 |
| m9-weibull-dist A144 | `WEIBULL.DIST(2,3.7,1,FALSE)` | 5.4560076923725256e-05 | 5.4560076923725154e-05 |
| m9-weibull-dist A146 | `WEIBULL.DIST(5,3.7,1,FALSE)` | 9.363099887555361e-166 | 9.363099887553769e-166 |
| m9-weibull-dist A159 | `WEIBULL.DIST(0.1,3.7,9,TRUE)` | 5.878989740656108e-08 | 5.8789897406560994e-08 |
| m9-weibull-dist A168 | `WEIBULL.DIST(5,3.7,9,FALSE)` | 0.07505415050895907 | 0.07505415050895915 |
| m9-weibull-dist A172 | `WEIBULL.DIST(30,3.7,9,FALSE)` | 4.605259708338221e-37 | 4.6052597083384175e-37 |
| t1048-weibull A8, B8 | `WEIBULL(10,3,8,FALSE)`, `WEIBULL.DIST(..)` | 0.08310360884023987 | 0.08310360884023976 |
| t1049-weibulldist A20 | `WEIBULL.DIST(10,1,1,FALSE)` | 4.539992976248477e-05 | 4.5399929762484854e-05 |
| t1049-weibulldist A22 | `WEIBULL.DIST(10,2,3,FALSE)` | 3.321186338840311e-05 | 3.321186338840322e-05 |

Lead, recorded in docs/excel-semantics/statistics.md "The far-tail density is less accurate than
exp(-t)". Compute t = (x/l)^k as exp(k*ln(x/l)). In a libm re-run over all 230 in-domain cells (x, k, l > 0 with a literal TRUE/FALSE cumulative; 116 cdf, 114 density):
- the cdf `-expm1(-t)` leaves no cell visibly off, and 94 of 116 are bit-exact, against 1 visibly off and 76 bit-exact for the
  engine's statrs form;
- the density `k/l*exp((k-1)*ln(x/l))*exp(-t)` leaves 4 visibly off (m9 A158, A168; t1048 A8, B8);
- `k/l^k*exp((k-1)*ln x)*exp(-t)` leaves 2 (t1048 A8, B8), against 17 for the engine's statrs form (17 + the cdf's 1 are the
  18 cells in the table).
- The same t also gives ironcalc-calc WEIBULL_DIST.xlsx E2 bit for bit (T-0978's line).

The fit is numeric, but the evidence is already in `eval/` (decision 0083: where Excel itself is inaccurate, its algorithm has to be reproduced).
Use the engine's own `ln`/`exp` kernels (numbers.md; fit lab, T-1523) before settling a form.
Extra candidate densities are cheap to test.

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): WEIBULL and WEIBULL.DIST values, and their overflow and underflow edges: `WEIBULL(1E+300,2.5,100,0.5)` and `WEIBULL(2,1E+5,0.5,TRUE)` are `#NUM!` where ours gives 1, `WEIBULL(0,1.5E+10,0.01,1)` is 0 where ours is `#NUM!`, `WEIBULL.DIST(0.1,10000000000.5,0.2,FALSE)` `#DIV/0!`. Booleans from cells are T-1849's.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 44 known-failure lines in eval/baseline/gen-m9.txt, 29 visibly off.
Examples:
- gen-m9/0bbab6206b5c9a07 Calc!A340 `=WEIBULL.DIST(100,1E+10,0.01,1)`: Excel #NUM!, ours 1.0
- gen-m9/329b177b5d5165ad Calc!A101 `=WEIBULL(1E+300,2.5,100,0.5)`: Excel #NUM!, ours 1.0
- gen-m9/5389dd4d5b5739d2 Calc!A378 `=WEIBULL.DIST(100,2.5,1.5E+10,TRUE)`: Excel 3.6288736930120994e-21, ours 3.628873693012116e-21
- gen-m9/5cebac1f02bf30f2 Calc!A336 `=WEIBULL(1.5,1.999,1.5E+10,(((-1)+42)>Data!E4))`: Excel 1.3637084610461536e-20, ours 1.3637084610461485e-20
- gen-m9/a63d1c7d6f96c56d Calc!A352 `=WEIBULL.DIST(Data!$A9,99999,99999,"FALSE")`: Excel #NUM!, ours 0.0
Cells:
- gen-m9/0072265d385d8ab2.case.json (1): Calc!A14
- gen-m9/0620505280d2ea6e.case.json (2): Calc!A61, A247
- gen-m9/0bbab6206b5c9a07.case.json (1): Calc!A340
- gen-m9/11eb3ab4f4f8104c.case.json (3): Calc!A85, A258, A320
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A360
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A101
- gen-m9/3b7990bd4e3307ba.case.json (1): Calc!A308
- gen-m9/3fb9cecf2c23169f.case.json (4): Calc!A17, A337, A342, A389
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A320
- gen-m9/44a8841352dc48e9.case.json (2): Calc!A256, A370
- gen-m9/46ac3ef5e61583ca.case.json (1): Calc!A339
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A378
- gen-m9/5395b8a427764a3f.case.json (2): Calc!A272, A288
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A50
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A336
- gen-m9/607e85cfee83a689.case.json (2): Calc!A158, A344
- gen-m9/6e18550e9fc9c1b9.case.json (1): Calc!A350
- gen-m9/7fd0a98183c6babf.case.json (1): Calc!A390
- gen-m9/873f30d2bf76f8db.case.json (2): Calc!A250, A364
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A197
- gen-m9/941c409532603d32.case.json (2): Calc!A18, A142
- gen-m9/9bc247733a1ca9fa.case.json (2): Calc!A108, A123
- gen-m9/a4090567758520df.case.json (1): Calc!A315
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A352
- gen-m9/c4635963fc926284.case.json (1): Calc!A46
- gen-m9/c7664e494e418395.case.json (2): Calc!A36, A226
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A248
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A114
- gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A35
- gen-m9/eccd03df42e40006.case.json (1): Calc!A96
- gen-m9/f82473275380882e.case.json (1): Calc!A356

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2256): these 44 known-failure lines (the T-2256 cells in Context) disappear from
      eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Every cell in the table is within half a unit of the 15th significant digit of Excel's value.
- [ ] No other WEIBULL.DIST or WEIBULL cell in m9-weibull-dist, t1048-weibull, t1049-weibulldist or
      ironcalc-calc WEIBULL_DIST.xlsx becomes visibly off. A cell that moves from bit-exact to
      4 ulp or less follows decision 0083's allowance and is listed in the Outcome.
- [ ] eval/baseline/ironcalc-calc.txt gains no line. If WEIBULL_DIST.xlsx E2 or E3 leaves it, the
      baseline is updated with `make eval-baseline` and T-0978 is told in the Outcome.
- [ ] The chosen form is recorded in statistics.md (the entry above moves from lead to rule, with
      the cells it matches).
- [ ] `make check` green; reviewers approve.

## Out of scope
- x = 0 densities and boolean arguments (T-2035: t1048-weibull A16/B16, t1049-weibulldist A27).
- Bit-exact parity on the whole sweep (T-0978, after this stage).

## Visible-parity stage
This whole task is the visible-parity stage (decision 0083). Its target is every cell it owns within
half a unit of the 15th significant digit of Excel's value. Any bit-level miss left after that
stays with T-0978.
