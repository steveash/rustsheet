---
id: T-0925
title: "Inverse normal (Wichura AS241) for NORM.S.INV/NORM.INV/LOGNORM.INV/CONFIDENCE.NORM (~120 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-1006, T-1746]
---

## Goal
NORM.S.INV, NORM.INV, LOGNORM.INV, CONFIDENCE.NORM (and the NORM-based part of CONFIDENCE.T inputs where applicable) return Excel's values, using one shared accurate inverse-normal routine.

## Context
Kernel map (T-1522, decision 0056): rank 11 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle-free: Excel's cached values are the exact quantiles, ours (statrs `Normal::inverse_cdf`, which goes through a lower-accuracy erf_inv) are off by ~1e-11.
- `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` `NORM.S.DIST NORM.S.INV!D2` `=NORM.S.INV(C2)` expected 1.3333330000000008, actual 1.3333329999747054; `D4` expected 0.8000000000000002 actual 0.7999999999297425. NORM.S.INV 35 cells, NORM.INV 29 cells (incl. 2 NumericMismatch).
- `ironcalc-calc/STATISTICAL/LOGNORM.xlsx` `LOGNORM.DIST LOGNORM.INV!F2` expected 4.000000000000003 actual 4.000000000091908 (31 cells).
- `ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx` CONFIDENCE.NORM 23 cells, all NumericUlp (`D2` expected 0.2771807648699355 actual 0.27718076486993554).
Code: `functions/statistical/{normal,log_normal}.rs`. Implement AS241 (PPND16) once, in the same module as the T-0924 kernel if that has landed (else a new `normal_kernel` module that T-0924 extends), and route all four functions through it. Where AS241 alone leaves ulp-level diffs, test whether one Newton/Halley refinement step against the T-0924 CDF reproduces the cached cells and keep what matches.

**Needs an oracle sweep (T-0018):** fitting this formulation needs more Excel points than the
cached workbooks give. Fit against the T-1006 sweep (`probes-numeric/m9-norm-s-inv`), never blind.

Triage T-0903 (2026-10-06 report): this task owns 106 baseline lines: NORM.S.INV 33, NORM.INV 19,
LOGNORM.INV 31 (all NORM_DIST.xlsx / LOGNORM.xlsx `*.INV` cells) and CONFIDENCE.NORM 23. The normal
CDF it inverts is not correctly rounded (`docs/excel-semantics/statistics.md`); its last digits are
T-0968, so an iteration against Excel's CDF needs T-0968 first.

gen-m9 sweep (gap triage 2026-10-23, T-2255). LOGNORM.INV, LOGINV, CONFIDENCE.NORM and CONFIDENCE, whose normal quantile is visibly off
(14 cells).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 14 known-failure lines in eval/baseline/gen-m9.txt, 14 formulas.
Examples:
- gen-m9/06b4e8f5de409d5e Calc!A343 `=LOGNORM.INV(1E-16,MAX(LN(1E+21),Data!A2:A7,Data!C8:C11),30)`: Excel 7.513666478165065e-87, ours 7.513666478164852e-87
Cells:
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A343
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A362
- gen-m9/4250f880ab640c49.case.json (2): Calc!A185, A232
- gen-m9/4e55500bb8ad3cd8.case.json (2): Calc!A75, A213
- gen-m9/530c636749a85209.case.json (1): Calc!A268
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A230
- gen-m9/7674c89c824aca43.case.json (1): Calc!A73
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A28
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A63
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A55
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A342
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A36

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: CONFIDENCE.NORM, CONFIDENCE, LOGINV and LOGNORM.INV last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 47 known-failure lines in eval/baseline/gen-m9.txt, 47 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A169 `=CONFIDENCE((MAX(Data!$B11,Data!E6:E8)^Data!$E10%),AGGREGATE(10,4,Data!B9:E9,Data!B4:D9),3)`: Excel 1020801326659.5261, ours 1020801326659.5262
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A169
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A199, A373
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A53
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A140
- gen-m9/27601819397dd56e.case.json (2): Calc!A70, A164
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A20
- gen-m9/2a18b2f5a4dc43ac.case.json (3): Calc!A156, A253, A278
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A49
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A4
- gen-m9/40425341c5034eed.case.json (1): Calc!A260
- gen-m9/530c636749a85209.case.json (1): Calc!A324
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A143
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A353
- gen-m9/623663cacea53bbc.case.json (1): Calc!A58
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A230
- gen-m9/7674c89c824aca43.case.json (1): Calc!A215
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A296
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A373
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A213, A386
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A224, A251
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A48, A231
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A176, A190
- gen-m9/a54c738c697617c3.case.json (2): Calc!A180, A228
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A170
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A395
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A181
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A105
- gen-m9/bc40d37cb4d8301d.case.json (2): Calc!A107, A204
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A117
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A84, A317
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A123
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A308
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A321
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A246
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A91
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A356

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): NORMINV, NORM.INV and NORMSINV last digits, none visibly off.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 11 known-failure lines in eval/baseline/gen-m9.txt, 0 visibly off.
Examples:
- gen-m9/097af1ca33638670 Calc!A283 `=NORM.INV(0.1,SEQUENCE(1),2.5)`: Excel -2.2038789138615016, ours -2.203878913861501
- gen-m9/17c110a53924beb8 Calc!A190 `=NORMSINV(0.2)`: Excel -0.8416212335729145, ours -0.8416212335729143
- gen-m9/cb61169f4c033d4b Calc!A22 `=NORMINV(Data!A12,1e-15,1.5)`: Excel -1.9223273483168997, ours -1.9223273483168999
- gen-m9/cb61169f4c033d4b Calc!A193 `=NORMSINV((Data!E6*MONTH(28931)))`: Excel -0.5244005127080408, ours -0.5244005127080407
- gen-m9/d8f44d556674c2fd Calc!A9 `=NORMINV(Data!A6,Data!E4,Data!B4)`: Excel -103903855.15465043, ours -103903855.15465039
Cells:
- gen-m9/097af1ca33638670.case.json (1): Calc!A283
- gen-m9/17c110a53924beb8.case.json (1): Calc!A190
- gen-m9/41248d1cb153c66e.case.json (1): Calc!A6
- gen-m9/5be29fa316931f05.case.json (1): Calc!A231
- gen-m9/607e85cfee83a689.case.json (2): Calc!A236, A397
- gen-m9/a4090567758520df.case.json (1): Calc!A336
- gen-m9/cb61169f4c033d4b.case.json (3): Calc!A22, A193, A321
- gen-m9/d8f44d556674c2fd.case.json (1): Calc!A9

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 25 known-failure lines (14 from T-2255, 11 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] All NumericClose/NumericMismatch lines for NORM.S.INV, NORM.INV, LOGNORM.INV disappear from `eval/baseline/ironcalc-calc.txt`; the CONFIDENCE.NORM ulp lines disappear or the PR documents which remain and why. Exact before/after counts in the PR; no new lines.
- [ ] No other call sites of statrs `inverse_cdf` for Normal (rg).
- [ ] Unit tests incl. p near 0/1 and p = 0.5; `make check` green.

## Out of scope
erf/CDF accuracy (T-0924); T.INV/GAMMA.INV/BETA.INV iterations.

## Fit result (T-1746)
Implement PPND16 (Horner; log(1-p) on the upper side) for |p-0.5| > 0.425: it is bit-exact on 611 of 616 tail cells. Centrally, keep statrs: nothing fitted there, and statrs is the best candidate. The hybrid is bit-exact on 775 of 1,024 sweep cells, against 341 today (`uv run rustsheet-fitlab norm-s-inv`; docs/excel-semantics/statistics.md, "NORM.S.INV is AS 241 PPND16 in the tails").

## Earlier attempt (draft PR #38, 2026-10-04; salvaged by T-0031)
Plain AS241 (PPND16) fixed 57 of ~120 cells (ironcalc-calc known failures 1632 -> 1580) but regressed
5 cells that statrs matched by luck: `CONFIDENCE.xlsx CONFIDENCE.NORM CONFIDENCE.T!D8`,
`LOGNORM.xlsx LOGNORM.INV!F72`, `NORM_DIST.xlsx NORM.INV!F28`, `NORM.S.INV!D11` (expected
-2.0000000000000004) and `D13` (expected exactly -3.0; AS241 gives -2.999999999999999). One Newton or
Halley step against the T-0924 CDF was worse (fixed 38, 9 regressions). Excel's last digits (e.g.
`NORM.S.INV(C2)` -> 1.3333330000000008) look like iteration against Excel's own CDF, so parity likely
needs that CDF bit-for-bit (several NORM.S.DIST cells, e.g. `NORM.S.INV!C3`, are still NumericUlp)
plus oracle probes. The oracle now exists: probe NORM.S.INV at p = NORM.S.DIST(k) for integer k first.

## Update (T-1916, 2026-10-18)
T-1916 made NORM.S.DIST/NORM.DIST bit-exact through the T-1778 kernels. Three NORM.INV/NORM.S.INV cells
(NORM_DIST.xlsx F4, F58, D13) then lost their lucky match with statrs, so T-1916 shipped the T-1746 tail half of
this task: `normal.rs` `ppnd16_tail` for |p - 0.5| > 0.425 in NORM.S.INV, NORM.INV and CONFIDENCE.NORM (statrs
stays centrally). With it ironcalc-calc lost 18 NORM.INV, 18 NORM.S.INV/NORM.S.DIST and 17 CONFIDENCE.NORM
known-failure lines and none regressed. What remains here: the central region (an iterative search on Excel's
NORM.S.DIST, statistics.md "NORM.S.INV is AS 241 PPND16 in the tails") and LOGNORM.INV, which still calls statrs.

## Update (T-0925 part 1, 2026-10-25)
Shipped:
- LOGNORM.INV and LOGINV are EXP of `normal::normal_quantile`, so they take the PPND16 tails. That fixes 20
  ironcalc-calc LOGNORM.xlsx lines and 17 gen-m9 lines, 11 of them among the 14 T-2255 visible-parity cells.
- The third acceptance criterion (no other statrs normal `inverse_cdf` call) is still open. T.INV's large-df
  series (`t_dist.rs`) keeps statrs' z. Moving it to the PPND16 hybrid moved 38 T.INV, TINV, T.INV.2T and
  CONFIDENCE.T cells, all already failing: 28 went up to 4 ulp further from Excel and 10 closer. There is no Excel
  evidence that T.INV builds on PPND16, so the change was reverted.

Across every suite, in-scope visibly-off cells fall from 11,309 to 11,276, and no cell newly fails.

The central region is still statrs. A bisection or Newton solve on Excel's own NORM.S.DIST, with every stop rule
tried, matches at most 68 of the 341 central cells, against statrs' 164 (statistics.md "NORM.S.INV is AS 241
PPND16 in the tails").

Still visibly off in gen-m9, all central (|p - 0.5| <= 0.425):
- 8d3588d8df0812df A28 `CONFIDENCE.NORM(FDIST(0.5,7,6),1E-300,26)` (11 ulp);
- 8f2ac25945af87da A63 `LOGNORM.INV(...)` (256 ulp);
- 8fec7b207f4dd753 A55 `LOGINV(0.2,250,30)` (78 ulp).

NORM_DIST.xlsx (41 lines) and CONFIDENCE.xlsx (26) are unchanged: their cells are central.

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 85 in total.
- NORM.S.INV: 65 cells (13-14 digits right 65, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1746-norm-s-inv-central.case.json` 38; `probes-numeric/t1759-norm-s-inv-roundtrip.case.json` 20. e.g. `NORM.S.INV(A35)` Excel -0.9944578832097497, ours -0.9944578832097531
- NORMSINV: 2 cells (13-14 digits right 2, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-norm-s-inv.case.json` 2. e.g. `NORMSINV(NORM.S.DIST(1.0,TRUE))` Excel 0.9999999999999996, ours 1.0000000000000004
- NORM.INV: 6 cells (13-14 digits right 6, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `ironcalc-calc/STATISTICAL/NORM_DIST.xlsx` 4; `probes-numeric/m9-norm-s-inv.case.json` 2. e.g. `_xlfn.NORM.INV(E8,B8,C8)` Excel 0.9999999999999996, ours 1.0000000000000004. Visibly off on main 5b8c2ae7:
  `NORM_DIST.xlsx` `NORM.DIST NORM.INV!F70` `NORM.INV(A70,B70,C70)`, Excel 0.9985762706156575, ours 0.9985762706156597
  (F8, F9 and F39 are off within 15 digits; T-3021)
- LOGNORM.INV: 12 cells (13-14 digits right 11, 10-12 1, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1543-inverse-cdf-tiny-p.case.json` 11; `ironcalc-calc/STATISTICAL/LOGNORM.xlsx` 1. e.g. `LOGNORM.INV(1E-20,0,1)` Excel 9.493291347224372e-05, ours 9.493291347224338e-05

Excel's centre is about 1e-15 from the true quantile while the engine is closer to the truth, so 15 digits needs Excel's centre formula; LOGNORM.INV still calls statrs `LogNormal::inverse_cdf` (log_normal.rs) and should move onto the same kernel.

## Cases added by the Phase 1 triage (T-0090, 2026-10-27)
NORM.S.INV is below the Phase 1 rate (45 visibly-off cells to fix at the 2026-10-27 eval), all in two probes-numeric cases no live task owned (T-0968, parked, named one). This task now owns both whole cases:
- probes-numeric/t1746-norm-s-inv-central (20 visibly-off cells)
- probes-numeric/t1759-norm-s-inv-roundtrip (20 visibly-off cells)
