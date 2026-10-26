---
id: T-2699
title: "HYPGEOM.DIST and HYPGEOMDIST: a sample below the smallest possible count is 0, not #NUM! (gen-m9, 13 cells)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
campaign: semantics
depends_on: [T-2700]
---

## Goal
HYPGEOM.DIST (pmf and cdf) and HYPGEOMDIST return 0 when sample_s is below number_sample - (number_pop - population_s),
the smallest count the draw can produce, as Excel does. Today they return `#NUM!`.

## Context
Cut from T-2429 (gap triage 2026-10-25, sonnet refill). This half of T-2429 needs no probe: every cell below shows the same
rule. T-2429 keeps the huge-count `#NUM!` and its unprobed threshold, and the text or negative `cumulative` cells. The
negative fractional counts went to T-2700.

Rule and evidence: docs/excel-semantics/statistics.md "HYPGEOMDIST: a sample below the possible range is 0; a negative or
huge count is #NUM! (T-2255)". The T-2436 probe showed the cdf follows the same rule: `HYPGEOM.DIST(1,3,6,7,TRUE)` is 0.

Code: `hyp_geom_dist_of` in crates/rustsheet-engine/src/functions/statistical/hypegeom.rs. The `valid` check folds
`sample_s >= max(0, number_sample + population_s - number_pop)` into the `#NUM!` conditions. Split it out:
- keep `#NUM!` for the other conditions (number_sample or population_s outside [0, number_pop], number_pop <= 0, a
  negative sample_s);
- return 0 for a sample below the lower bound, for both pmf and cdf.

The existing branch above min(number_sample, population_s) (density 0, cdf 1) stays as it is.

Excel cells (excel-graph; known failures in eval/baseline/gen-m9.txt):

| cell | formula | Excel | ours |
|---|---|---|---|
| gen-m9/4250f880ab640c49 Calc!A339 | `HYPGEOMDIST(2,5,10,11)` | 0 | #NUM! |
| gen-m9/4bad303d32cc3f46 Calc!A267 | `HYPGEOMDIST(0,5,6,9)` | 0 | #NUM! |
| gen-m9/4e55500bb8ad3cd8 Calc!A318 | `HYPGEOMDIST(0,1,2,2)` | 0 | #NUM! |
| gen-m9/8f2ac25945af87da Calc!A218 | `HYPGEOMDIST(3,4,7,7)` | 0 | #NUM! |
| gen-m9/9cca17438a7e6f8a Calc!A340 | `HYPGEOMDIST(1,4,7,9)` | 0 | #NUM! |
| gen-m9/c2974c061384fdd9 Calc!A81 | `HYPGEOMDIST(2,4,4,5)` | 0 | #NUM! |
| gen-m9/c468f5046aadfbbc Calc!A85 | `HYPGEOMDIST(3,5,5,6)` | 0 | #NUM! |
| gen-m9/c6cfda6bd10cae93 Calc!A31 | `HYPGEOMDIST(0,2,5,6)` | 0 | #NUM! |
| gen-m9/05d44f6dd6cfc567 Calc!A54 | `HYPGEOMDIST(SUMIF(Data!B4:B9,"a*",Data!C2:D8),2,8,9)` | 0 | #NUM! |
| gen-m9/9cca17438a7e6f8a Calc!A96 | `HYPGEOMDIST(Data!B$8,MIN(Data!E11:E11),5,Data!E$11)` | 0 | #NUM! |
| gen-m9/a7bf0d8d81d33ba6 Calc!A315 | `HYPGEOM.DIST(1,2,Data!$D$3,7,ISFORMULA(Data!$B9))` (`HYPGEOM.DIST(1,2,7,7,FALSE)`) | 0 | #NUM! |
| gen-m9/0a9226016e8f5f33 Calc!A26 | `CHIDIST(TIME(11,HYPGEOMDIST(2,5,9,11),1),9)` | 0.9999790730352063 | #NUM! |
| gen-m9/35e8168aae2b00f8 Calc!A271 | `KURT(Data!D10:D11,YEAR(HYPGEOM.DIST(1,3,6,7,TRUE)))` | #DIV/0! | #NUM! |

The last two come from the T-2436 sub-expression probe (probes/t2436/0a9226016e8f5f33 and 35e8168aae2b00f8): there the inner
HYPGEOM call is Excel's 0, and the outer call then gives Excel's value.

Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A54
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A26
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A271
- gen-m9/4250f880ab640c49.case.json (1): Calc!A339
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A267
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A318
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A218
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A96, A340
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A315
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A81
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A85
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A31

T-2429 (2026-10-25) probes more impossible samples: probes/t2429-hypgeom-huge-count A5, A6, A11, A12, ... (`HYPGEOMDIST(1,N,3,N)` and `HYPGEOMDIST(1,3,N,N)`, Excel 0, ours `#NUM!`, for N from 202770 to 1E9) and probes/t2429-hypgeom-threshold A3, A4, A7, A8, ... (N to 2147483646.9) are known failures in eval/baseline/probes.txt that this rule fixes.

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2436): these 13 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline (the probes-numeric t1048/t1049
      hypgeomdist cells included).
- [ ] The T-2436 probe's 18 cells over the same two calls leave eval/baseline/probes.txt:
      probes/t2436/0a9226016e8f5f33 Calc!A76, A91, A106, T76, T91, T106, U76, U91, U106 (`HYPGEOMDIST(2,5,9,11)` = 0
      and its TIME and CHIDIST), and probes/t2436/35e8168aae2b00f8 Calc!A196, A211, A226, T196, T211, T226, U196, U211,
      U226 (`HYPGEOM.DIST(1,3,6,7,TRUE)` = 0 and its YEAR and KURT). Any other probes/t2436 cell this change fixes leaves
      the baseline too.
- [ ] Unit tests: `HYPGEOMDIST(3,5,5,6)` and `HYPGEOM.DIST(1,3,6,7,TRUE)` are 0. The existing hypegeom.rs tests of
      the other `#NUM!` conditions still pass.
- [ ] statistics.md's T-2255 entry gets an "Ours:" line for the impossible sample naming this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A huge count (about 1E15) and its threshold, and a text or negative `cumulative` (T-2429).
- Negative fractional counts (T-2700).
- Last digits (T-0977, T-2481) and large-population cdfs (T-1896).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type or error (none is a last-digit difference), so the task is in
the campaign at the `semantics` stage. Its target is that every cell it owns has Excel's type and error, or a value within
half a unit of the 15th significant digit; the visible-parity criterion under Acceptance criteria states it.
