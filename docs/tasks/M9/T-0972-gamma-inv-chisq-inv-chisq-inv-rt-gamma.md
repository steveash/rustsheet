---
id: T-0972
title: "GAMMA.INV/CHISQ.INV/CHISQ.INV.RT: gamma quantile parity (39 known failures)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: S
tier: opus
campaign: visible
depends_on: [T-0971, T-1045]
---

## Goal
GAMMA.INV, CHISQ.INV and CHISQ.INV.RT return Excel's values from one gamma-quantile routine built
on the T-0971 incomplete-gamma kernel.

## Context
Kernel map (T-1522, decision 0056): rank 10 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Oracle targets: probes-numeric cases `m9-gamma-dist-a*`, `m9-chisq-dist`, `m9-chisq-dist-rt`, `m9-gamma-inv`, `m9-chisq-inv`, `m9-chisq-inv-rt` (T-1045).
Triage T-0903. Kernel: statrs' default `ContinuousCDF::inverse_cdf` for `Gamma` (bisection from
[1,2] widened by halving/doubling, 8 bisection steps, then Newton on statrs' CDF), reached through
`functions/statistical/gamma.rs` `fn_gamma_inv` and `functions/statistical/chisq.rs` `fn_chisq_inv`,
`fn_chisq_inv_rt` (which inverts `1.0 - p`).
39 baseline lines in `eval/baseline/ironcalc-calc.txt` (11 NumericClose, 28 NumericUlp):
- `ironcalc-calc/STATISTICAL/CHISQ.xlsx` `Sheet1`: I2, J2
- `ironcalc-calc/STATISTICAL/GAMMA.xlsx` `GAMMA.DIST GAMMA.INV`: F2, F3, F4, F5, F6, F7, F8(C), F9, F10(C), F11, F19, F21, F22(C), F24, F34, F39, F41, F42, F43, F44, F45, F46, F47, F50, F51, F53, F61, F62, F66(C), F67(C), F68(C), F69(C), F70(C), F74(C), F75, F76(C), F77(C)
(C = NumericClose, unmarked = NumericUlp.)

Also in probes (unchanged by T-1872, which lifted these functions): probes/stat-argument-modes H140 `CHIINV(0.4,5)` and
H148 `CHISQ.INV.RT(0.4,5)` expected 5.131867074401821, actual 5.131867074401809; H144 `CHISQ.INV(0.4,5)` expected
3.6554996231415857, actual 3.655499623141595 (NumericClose); probes/stat-quantile-lift J35
`ROUND(INDEX(CHISQ.INV.RT(0.4,G35:G36),2),12)` and J37 (CHIINV), df 10: expected 10.473236231396, actual
10.473236231395 (our scalar quantile is 10.473236231395429).

Examples (expected vs actual):
- `GAMMA.INV(GAMMA.DIST(10,9,2,TRUE),9,2)` F2 (14 lines with this pair): expected 9.999999999999998, actual
  9.999999999999993. Excel does not round-trip to 10 (`docs/excel-semantics/statistics.md`).
- F74: expected 61.91422671617931, actual 61.91422671628383 (1.7e-12); F77: expected 1999.3333728539303,
  actual 1999.3333728539678.
- `CHISQ.INV.RT(CHISQ.DIST.RT(12.3,...))` J2: expected 12.3, actual 12.299999999999994.

Hypothesis (the oracle is the judge): Excel solves P(a, x/b) = p iteratively on its own GAMMA.DIST
kernel (T-0971); CHISQ.INV.RT solves on the upper tail. The round-trip residue (9.999999999999998)
points at a tolerance-based stop. **Needs an oracle sweep (T-0018):** fit against the T-1045
incomplete-gamma sweep (family 6), never blind.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/4df1b22d3ed63f58 Calc!A9 `=CHIINV(T.DIST(0.1,7,Data!D1),2)` (D1 = 0.2, cumulative): Excel 1.2382105318705625, engine 1.2382105318705663. CHIINV with 2 degrees of freedom is -2*LN(p) in closed form; the miss may be T.DIST's (T-0969) as much as CHIINV's.

gen-m9 sweep (gap triage 2026-10-23, T-2255). CHISQ.INV, CHIINV, CHISQ.INV.RT, GAMMA.INV and GAMMAINV at moderate probabilities and
shapes where ours is visibly off (64 cells).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 64 known-failure lines in eval/baseline/gen-m9.txt, 64 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A27 `=GAMMA.INV(0.999999,0.01,2)`: Excel 14.302506956169765, ours 14.302506956241972
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A27
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A276
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A167
- gen-m9/165db58d1540f014.case.json (1): Calc!A375
- gen-m9/27601819397dd56e.case.json (1): Calc!A218
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A13, A158
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A372
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A315
- gen-m9/40425341c5034eed.case.json (1): Calc!A247
- gen-m9/4250f880ab640c49.case.json (1): Calc!A371
- gen-m9/4bad303d32cc3f46.case.json (1): Calc!A349
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A125
- gen-m9/58e3253bbd3a7a82.case.json (2): Calc!A37, A284
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A15, A85
- gen-m9/623663cacea53bbc.case.json (1): Calc!A159
- gen-m9/7674c89c824aca43.case.json (1): Calc!A152
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A79
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A228
- gen-m9/8d3588d8df0812df.case.json (1): Calc!A232
- gen-m9/8f2ac25945af87da.case.json (1): Calc!A64
- gen-m9/8fec7b207f4dd753.case.json (2): Calc!A103, A173
- gen-m9/984cba67bbe6972b.case.json (4): Calc!A41, A85, A104, A272
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A61
- gen-m9/a54c738c697617c3.case.json (1): Calc!A89
- gen-m9/a5828d3925575713.case.json (3): Calc!A96, A243, A326
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A261
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A141
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A61
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A42
- gen-m9/bbdfc06cae0e0d71.case.json (2): Calc!A113, A205
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A163
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A264
- gen-m9/c18dbf21f11093f3.case.json (2): Calc!A71, A199
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A50, A58
- gen-m9/c468f5046aadfbbc.case.json (3): Calc!A6, A67, A317
- gen-m9/c6cfda6bd10cae93.case.json (1): Calc!A266
- gen-m9/caa1eed48b1883fe.case.json (2): Calc!A343, A377
- gen-m9/cde551ea787f713a.case.json (1): Calc!A134
- gen-m9/d8266bc15533702a.case.json (4): Calc!A29, A247, A248, A334
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A168
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A334
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A5, A286
- gen-m9/f2bd5f380ec002bf.case.json (2): Calc!A146, A293
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A310

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: CHIINV, CHISQ.INV, CHISQ.INV.RT, GAMMAINV and GAMMA.INV last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 121 known-failure lines in eval/baseline/gen-m9.txt, 121 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A16 `=CHISQ.INV.RT(0.2,1.999)`: Excel 1.6423744151498165, ours 1.6423744151498179
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (3): Calc!A16, A32, A215
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A61
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A337
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A30
- gen-m9/165db58d1540f014.case.json (1): Calc!A53
- gen-m9/27601819397dd56e.case.json (1): Calc!A47
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A31, A283
- gen-m9/285d87cea1aaa73c.case.json (4): Calc!A132, A139, A200, A261
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A299
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A196, A378
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A175
- gen-m9/40425341c5034eed.case.json (3): Calc!A17, A351, A385
- gen-m9/4250f880ab640c49.case.json (3): Calc!A66, A213, A310
- gen-m9/4ab2ba98d26c8a5e.case.json (2): Calc!A39, A85
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A209, A289
- gen-m9/530c636749a85209.case.json (3): Calc!A50, A228, A365
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A34, A342
- gen-m9/58e3253bbd3a7a82.case.json (2): Calc!A8, A267
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A268
- gen-m9/623663cacea53bbc.case.json (2): Calc!A238, A299
- gen-m9/7674c89c824aca43.case.json (2): Calc!A106, A336
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A9, A327
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A142
- gen-m9/85f48d5cdfb24555.case.json (3): Calc!A22, A327, A369
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A90, A373
- gen-m9/8d3588d8df0812df.case.json (3): Calc!A116, A289, A293
- gen-m9/8dd2312c96effb1b.case.json (4): Calc!A40, A232, A246, A293
- gen-m9/8f2ac25945af87da.case.json (2): Calc!A25, A132
- gen-m9/8fec7b207f4dd753.case.json (4): Calc!A80, A82, A143, A265
- gen-m9/984cba67bbe6972b.case.json (6): Calc!A90, A118, A132, A139, A209, A286
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A36
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A59, A80
- gen-m9/a54c738c697617c3.case.json (1): Calc!A310
- gen-m9/a5828d3925575713.case.json (2): Calc!A4, A142
- gen-m9/a612e89321a6df9c.case.json (3): Calc!A81, A307, A353
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A132
- gen-m9/a790a4d4a6a82756.case.json (3): Calc!A20, A104, A257
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A382
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A164, A291
- gen-m9/b4173fe25109a238.case.json (3): Calc!A69, A118, A220
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A306
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A275
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A197
- gen-m9/bcb24385baafb2f0.case.json (3): Calc!A88, A271, A386
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A349
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A119, A297
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A250
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A59, A303
- gen-m9/cde551ea787f713a.case.json (2): Calc!A209, A256
- gen-m9/e6cf3b20a014e6a3.case.json (2): Calc!A98, A159
- gen-m9/ecc87f80077959e0.case.json (7): Calc!A133, A154, A224, A231, A239, A302, A322
- gen-m9/ee2497a5e683c5d9.case.json (4): Calc!A56, A189, A281, A378
- gen-m9/f2bd5f380ec002bf.case.json (2): Calc!A17, A201
- gen-m9/f34f66fb7f81e673.case.json (3): Calc!A102, A172, A240
- gen-m9/fd9fa22d212080d6.case.json (2): Calc!A249, A359
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A61

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): CHISQ.INV.RT passed through NORMSDIST (visibly off).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 visibly off.
Examples:
- gen-m9/73f053591a54bc43 Calc!A108 `=NORMSDIST(MOD(SUMIF(Data!A10:D12,"",Data!B4:E12),CHISQ.INV.RT(0.975,5)))`: Excel 0.6004864351048662, ours 0.6004864351048611
Cells:
- gen-m9/73f053591a54bc43.case.json (1): Calc!A108

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 65 known-failure lines (64 from T-2255, 1 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] All 11 NumericClose lines above leave that class, at least 20 of the 39 lines leave
      `eval/baseline/ironcalc-calc.txt`, no new lines in any baseline; remaining lines listed in the PR.
- [ ] No statrs `inverse_cdf` left for Gamma/ChiSquared (`rg inverse_cdf`); bounded iteration on
      untrusted input (huge shape/scale, p near 0 and 1).
- [ ] `make check` green; reviewers approve.

## Out of scope
The CDF kernel (T-0971); legacy GAMMAINV/CHIINV evidence (T-1047, T-1048).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 236 in total.
- GAMMA.INV: 87 cells (13-14 digits right 58, 10-12 2, 6-9 14, fewer or 0 vs nonzero 13); most in `probes-numeric/m9-gamma-inv.case.json` 53; `ironcalc-calc/STATISTICAL/GAMMA.xlsx` 21. e.g. `GAMMA.INV(0.5,9,1)` Excel 8.668951184370373, ours 8.668951184370435
- GAMMAINV: 4 cells (13-14 digits right 4, 10-12 0, 6-9 0, fewer or 0 vs nonzero 0); most in `probes-numeric/t1048-gammainv.case.json` 4. e.g. `GAMMAINV(0.5,50,2)` Excel 99.33412923598846, ours 99.33412923598982
- CHISQ.INV: 44 cells (13-14 digits right 26, 10-12 2, 6-9 9, fewer or 0 vs nonzero 7); most in `probes-numeric/m9-chisq-inv.case.json` 28; `probes-numeric/t1543-inverse-cdf-tiny-p.case.json` 8. e.g. `CHISQ.INV(0.4,5)` Excel 3.6554996231415857, ours 3.655499623141595
- CHISQ.INV.RT: 73 cells (13-14 digits right 56, 10-12 4, 6-9 11, fewer or 0 vs nonzero 2); most in `probes-numeric/m9-chisq-inv-rt.case.json` 34; `probes-numeric/m9-legacy-chiinv.case.json` 26. e.g. `CHISQ.INV.RT(0.4,5)` Excel 5.131867074401821, ours 5.131867074401809
- CHIINV: 28 cells (13-14 digits right 22, 10-12 2, 6-9 4, fewer or 0 vs nonzero 0); most in `probes-numeric/m9-legacy-chiinv.case.json` 26; `probes/stat-argument-modes.case.json` 1. e.g. `CHIINV(0.4,5)` Excel 5.131867074401821, ours 5.131867074401809

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.

## Outcome (2026-10-28, owner-unblocked)
The baseline was recorded with `update-baseline --allow-regressions` under decision 0083's allowance as amended by
**decision 0144**, with the owner's approval in session https://claude.ai/code/session_0179K7WDtouX929jBuhJCWHr. Measured on
the branch merged with `main` (3fe509e8):
- gated: 28 lines fixed (gen-m9 23, ironcalc-calc 5); 49 cells regressed, all numeric (38 at 1 ulp, 9 at 2, 2 at 3);
- every suite, against a `main` build: 90 other cells newly fail within the 15th digit; in-scope visibly-off cells
  101,602 -> 101,575;
- five cells newly visibly off, accepted by decision 0144: gen-m9 ecc87f80077959e0 A168 and 146b46e74f2dac3b A167,
  ironcalc-calc GAMMA.xlsx F21, probes-numeric m9-chisq-inv A27 and t1049-chisqinv A19.

Follow-up **T-3016** owns those five cells and lists all 49 regressed cells with their values.

Not done, so the task stays open (status todo):
- Criterion 1: of the 65 owned gen-m9 lines, 17 now pass, 42 are within the 15th digit and 6 are still visibly off.
- Criterion 2 is not re-measured on this merged tree.
- Criterion 3 is not met: `incomplete_gamma/mod.rs` still keeps the ported statrs search (`inverse_cdf_within`) as the
  refinement's start.
