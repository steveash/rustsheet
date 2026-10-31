---
id: T-3235
title: "HYPGEOM.DIST and HYPGEOMDIST last digits once T-2481 has Excel's pmf (113 cells within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2481]
defer: phase-1b
---

## Goal
HYPGEOM.DIST and HYPGEOMDIST match Excel to the bit on the 113 cells listed below. All of them are already within half
a unit of Excel's 15th significant digit, and no open task lists any of them. T-2481 covers only the cells where Excel
is visibly off the exact rational, so this is its Phase 1b sibling.

## Context
- Gap triage 2026-10-30 (`make eval` at e1255db9, excel-graph evidence). Each cell has HYPGEOM.DIST (48 cells) or
  HYPGEOMDIST (65 cells) as its only non-glue call. By suite: gen-m9 60, probes-numeric 47, probes 6.
- Ulp distance, ours against Excel's: 1 ulp 36, 2 ulp 35, 3 ulp 15, 4 ulp 11, 5 ulp 11, 6 ulp 3, 12 ulp 2. The
  verdicts are 97 `NumericUlp` and 16 `NumericClose`, and none is visibly off.
- Since T-0977 the engine returns the exact rational rounded once (`hypergeometric_walk` in
  crates/rustsheet-engine/src/functions/statistical/discrete.rs; hypegeom.rs). Excel is a few ulp off it, and Excel
  is always right. Checked with Python fractions, Excel against ours (ours is the exact value):
  - probes-numeric/t1048-hypgeomdist Calc!A6 and B6 `=HYPGEOMDIST(10,20,40,100)`: 0.11923605235455972 against
    0.11923605235455989 (12 ulp).
  - probes-numeric/t1049-hypgeomdist Calc!A6 `=HYPGEOM.DIST(0,3,5,12,FALSE)`: 0.15909090909090923 against
    0.1590909090909091 = 7/44 (5 ulp).
  - probes-numeric/t1049-hypgeomdist Calc!A1 `=HYPGEOM.DIST(1,4,8,20,TRUE)`: 0.46542827657378727 against
    0.46542827657378744. Excel's value is the double sum of its own two pmf terms.
  - probes/t2429-hypgeom-threshold Calc!A5 `=HYPGEOMDIST(1,2,3,1500000000)`: 3.9999999946666614e-09 against
    3.999999994666666e-09 (6 ulp). This is a huge population.
- The rule and what has been ruled out: docs/excel-semantics/statistics.md "HYPGEOM.DIST is not exact for small
  integer arguments", including the T-2742 sweep. In that sweep only 286 of 1,442 pmf cells are exact, the pmf is not
  symmetric in n and K, and HYPGEOMDIST shares the kernel. Excel's cdf is built from its own pmf terms, so once
  T-2481 pins the pmf formulation, most of these cells should follow from it.
- What this task adds: score T-2481's formulation on these 113 cells. Fit whatever is left: the cdf summation order,
  and the huge-population path of the six probes cells (populations 1E7 to 2147483646), which may differ from the
  small-N one. Numeric fitting, so opus. The evidence is T-2742's small-N sweep (done) plus
  the cases below. If the huge-population cells cannot be separated on them, do not fit blind: first file a
  research-lane sweep of HYPGEOMDIST(1,2,3,N) over N from 1E6 to 2^31, and depend on it.

### Cells owned
Each cell below has HYPGEOM.DIST or HYPGEOMDIST as its only non-glue call:
- probes-numeric/t1049-hypgeomdist.case.json (29): Calc!A1, A2, A6, A8, A9, A11, A12, A19, A20, A21, A22, A23, A24, A25, A26, A27, A28, A29, A30, A31, A32, A37, A41, A51, A52, A53, A54, A58, A59
- probes-numeric/t1048-hypgeomdist.case.json (18): Calc!A1, A3, A4, A5, A6, A7, A9, A18, A19, B1, B3, B4, B5, B6, B7, B9, B18, B19
- probes/t2429-hypgeom-threshold.case.json (4): Calc!A5, A13, A17, A21
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A205, A297, A389
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A64, A308
- gen-m9/320d51765af9b6b7.case.json (2): Calc!A133, A344
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A309, A370
- gen-m9/40425341c5034eed.case.json (2): Calc!A44, A366
- gen-m9/4ab2ba98d26c8a5e.case.json (2): Calc!A94, A140
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A183, A324
- gen-m9/7674c89c824aca43.case.json (2): Calc!A18, A340
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A202, A386
- gen-m9/85f48d5cdfb24555.case.json (2): Calc!A331, A391
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A82, A161
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A274, A320
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A20, A142
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A153, A275
- gen-m9/cde551ea787f713a.case.json (2): Calc!A132, A193
- gen-m9/f34f66fb7f81e673.case.json (2): Calc!A37, A267
- gen-m9/fd9fa22d212080d6.case.json (2): Calc!A39, A202
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A85, A320
- probes/t2429-hypgeom-huge-count.case.json (2): Calc!A13, A19
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A359
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A168
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A234
- gen-m9/165db58d1540f014.case.json (1): Calc!A176
- gen-m9/27601819397dd56e.case.json (1): Calc!A134
- gen-m9/2801c1c11c21e134.case.json (1): Calc!A147
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A373
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A164
- gen-m9/4250f880ab640c49.case.json (1): Calc!A34
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A374
- gen-m9/530c636749a85209.case.json (1): Calc!A65
- gen-m9/623663cacea53bbc.case.json (1): Calc!A31
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A383
- gen-m9/a5828d3925575713.case.json (1): Calc!A65
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A119
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A278
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A290
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A17
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A207
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A146
- gen-m9/d8266bc15533702a.case.json (1): Calc!A363
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A356
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A374

## Acceptance criteria
- [ ] These known-failure lines disappear from the baselines, with no new ones in any baseline (`make eval-baseline`,
      `make eval-check`: 0 regressions):
      - the 60 gen-m9 lines from eval/baseline/gen-m9.txt;
      - the 6 probes lines from eval/baseline/probes.txt.
- [ ] probes-numeric is ungated, so its 47 owned cells are scored from `make eval`'s report.json: they pass
      bit-exact. Or, under the decision 0056 time box, the Outcome records the formulations tried with their bit-exact
      counts, and the open cells move to a follow-up.
- [ ] statistics.md's HYPGEOM.DIST entry states the cdf summation and the huge-population rule found, citing these
      cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The pmf formulation itself and the visibly-off HYPGEOM cells (T-2481).
- HYPGEOM's per-call cap and the #NUM! population bound (statistics.md T-1873, T-2255).
