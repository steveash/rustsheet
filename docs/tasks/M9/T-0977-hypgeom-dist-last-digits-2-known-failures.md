---
id: T-0977
title: "HYPGEOM.DIST last digits (2 known failures)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-1048, T-1049]
---

## Goal
HYPGEOM.DIST (density and cumulative) reproduces Excel's last digits.

## Context
Kernel map (T-1522, decision 0056): rank 17 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Triage T-0903. Kernel: `statrs::distribution::Hypergeometric` (`pmf` = ratio of `factorial::binomial`
values; `cdf` = sum of `exp(ln_binomial + ln_binomial - ln_binomial)` terms), called from
`functions/statistical/hypegeom.rs` `fn_hyp_geom_dist`.
2 baseline lines in `eval/baseline/ironcalc-calc.txt`, both NumericClose:
- `ironcalc-calc/STATISTICAL/HYPERGEOM_DIST.xlsx` `Sheet1`: F2 `HYPGEOM.DIST(1.8,4.2,8.2,22,TRUE)`: expected
  0.5349282296650714, actual 0.5349282296650703; F3 `HYPGEOM.DIST(1.8,4.2,8.2,18.34,FALSE)`: expected
  0.31372549019607815, actual 0.3137254901960784.
Excel is *not* the exact rational here (16/51 = 0.3137254901960784, which statrs already returns;
559/1045 = 0.5349282296650718), and neither a log-gamma nor a log-factorial exp-sum form reproduces
both cells (`docs/excel-semantics/statistics.md`, "HYPGEOM.DIST is not exact for small integer
arguments").
**Needs an oracle sweep (T-0018):** two cells cannot pin a formulation; fit against the
HYPGEOM.DIST evidence in T-1049 and HYPGEOMDIST evidence in T-1048, never blind. Candidates to try: pmf
via the binomial recurrence starting from the mode, pmf via `exp` of a Stirling-series / lgamma
difference, CDF as ascending vs descending sums, CDF via the complement when k is above the mean.

gen-m9 sweep (gap triage 2026-10-23, T-2255). HYPGEOM.DIST and HYPGEOMDIST values visibly off (22 cells, e.g. `HYPGEOMDIST(0,3,3.14159,
202769)` Excel 0.9999556149548074, ours 0.9999556141511747, the large-population cdf T-1896 also touches).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 22 known-failure lines in eval/baseline/gen-m9.txt, 22 formulas.
Examples:
- gen-m9/0a9226016e8f5f33 Calc!A93 `=HYPGEOM.DIST(0,4,1,14,OR(CELL("row",Data!D2)))`: Excel 0.7142857142857143, ours 0.7142857142857174
Cells:
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A93
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A87
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A374
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A140
- gen-m9/623663cacea53bbc.case.json (1): Calc!A258
- gen-m9/7674c89c824aca43.case.json (1): Calc!A140
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A297
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A161
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A20
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A373
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A49
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A377
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A290
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A394
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A385
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A392
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A66
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A198
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A267, A310
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A202
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A320

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: HYPGEOMDIST and HYPGEOM.DIST last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 65 known-failure lines in eval/baseline/gen-m9.txt, 65 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A359 `=HYPGEOMDIST(4,5,4,13)`: Excel 0.00699300699300699, ours 0.006993006993006993
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A359
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A64, A308
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A168
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A349
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A234
- gen-m9/165db58d1540f014.case.json (1): Calc!A176
- gen-m9/27601819397dd56e.case.json (1): Calc!A134
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A86, A147
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A373
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A164
- gen-m9/320d51765af9b6b7.case.json (2): Calc!A133, A344
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A309, A370
- gen-m9/40425341c5034eed.case.json (2): Calc!A44, A366
- gen-m9/4250f880ab640c49.case.json (1): Calc!A34
- gen-m9/4ab2ba98d26c8a5e.case.json (3): Calc!A41, A94, A140
- gen-m9/530c636749a85209.case.json (1): Calc!A65
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A183, A324
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A205, A297, A389
- gen-m9/623663cacea53bbc.case.json (1): Calc!A31
- gen-m9/7674c89c824aca43.case.json (2): Calc!A18, A340
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A202, A386
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A281
- gen-m9/85f48d5cdfb24555.case.json (2): Calc!A331, A391
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A82
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A363
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A383
- gen-m9/a5828d3925575713.case.json (1): Calc!A65
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A119
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A94, A278
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A11
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A274, A320
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A109
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A17
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A20, A142
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A207
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A153, A275
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A146
- gen-m9/cde551ea787f713a.case.json (2): Calc!A132, A193
- gen-m9/d8266bc15533702a.case.json (1): Calc!A363
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A160
- gen-m9/f2bd5f380ec002bf.case.json (2): Calc!A52, A374
- gen-m9/f34f66fb7f81e673.case.json (2): Calc!A37, A267
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A39
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A85, A177

T-2429 (2026-10-25) adds last-digit known failures at large populations in eval/baseline/probes.txt: probes/t2429-hypgeom-huge-count A1, A7, A13, A19, A25 and probes/t2429-hypgeom-threshold A1, A5, A9, A13, A17, A21, A25 (e.g. `HYPGEOMDIST(1,2,3,2147483646)`: Excel 2.7939677238464347E-09, ours 2.7939677238464355E-09).

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 22 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Both lines leave `eval/baseline/ironcalc-calc.txt`, the T-1049 HYPGEOM.DIST probe cells pass, no
      new lines in any baseline.
- [ ] Bounded loops for huge populations (no unbounded allocation or iteration on untrusted input).
- [ ] The statistics.md entry is updated with the matched formulation.
- [ ] `make check` green; reviewers approve.

## Out of scope
BINOM/NEGBINOM/POISSON (T-0927).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 23 in total.
- HYPGEOM.DIST: 22 cells (13-14 digits right 15, 10-12 4, 6-9 2, fewer or 0 vs nonzero 1); most in `probes-numeric/t1049-hypgeomdist.case.json` 19; `ironcalc-calc/STATISTICAL/HYPERGEOM_DIST.xlsx` 1. e.g. `HYPGEOM.DIST(1,4,8,20,2)` Excel 0.46542827657378727, ours 0.46542827657378827
- HYPGEOMDIST: 1 cells (13-14 digits right 1, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1048-hypgeomdist.case.json` 1. e.g. `HYPGEOMDIST(10,20,40,100)` Excel 0.11923605235455972, ours 0.1192360523545622

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.

## Outcome
HYPGEOM.DIST and HYPGEOMDIST are the exact value rounded once: `discrete.rs` `hypergeometric_walk` starts at the
support's lowest term (a product of min(n, K) or N - max(n, K) ratios) and steps by the exact ratio of consecutive terms,
in double-double with the binomial walk's scaling, up to 2^12 steps and a population of 2^52; statrs' pmf and the capped
cdf port stay beyond. HYPGEOMDIST is now charged 2^13 an element (16,384 a lifted call, a DEVIATION in statistics.md).

Eval (against main 2026-10-23): gen-m9 known failures 7028 -> 7017, probes 2026 -> 2025, 0 regressions. 42 cells are no
longer visibly off and none newly are: 20 of the 22 gen-m9 visible-parity cells above are now within 15 digits (18 of
them newly), and so are every HYPGEOM.DIST cdf
in t1049-hypgeomdist (statrs was up to 1.5E-10 off at a population of 1E5) and `HYPGEOM.DIST(1,4,8,2E9,FALSE)` (statrs
7 digits right). The ironcalc criterion (F2, F3) is not met: both are now within 15 digits but Excel's last bits are not
the exact value's.

Not met here, moved to T-2481: 2 of the 22 gen-m9 cells (ee2497a5e683c5d9 A310, 60b0b6e69ad35a45 A140), plus
f2bd5f380ec002bf A52 and t1049-hypgeomdist A10, A14, A16, where Excel's own pmf is visibly off the exact value; Excel's pmf formulation
is still open (statistics.md "HYPGEOM.DIST is not exact for small integer arguments" lists what was ruled out: 9!
log-factorial summation orders, COMBIN ratios, products of ratios, Loader's dhyper). Excel's cdf is the sum of its own
pmf terms, so that formulation also fixes the bit-level cells.
