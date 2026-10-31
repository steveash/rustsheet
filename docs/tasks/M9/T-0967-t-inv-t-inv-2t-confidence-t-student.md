---
id: T-0967
title: "T.INV/T.INV.2T/CONFIDENCE.T: Student-t quantile parity (89 known failures)"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-0969, T-1043, T-2511]
---

## Goal
T.INV, T.INV.2T and CONFIDENCE.T return Excel's values from one Student-t quantile routine built on
the T-0969 CDF kernel.

## Context
Kernel map (T-1522, decision 0056): rank 5 in `docs/research/numeric-kernel-map.md`; fit with the fit lab (T-1523).
Triage T-0903. Kernel: `statrs::distribution::StudentsT::inverse_cdf`, which calls statrs'
`inv_beta_reg(df/2, 0.5, 2p)` and back-transforms (`sqrt(df(1-y)/y)`), losing precision near p = 0.5.
Code: `functions/statistical/t_dist.rs` (`fn_t_inv`, `fn_t_inv_2t`), `functions/statistical/normal.rs`
(`fn_confidence_t`, `t_dist.inverse_cdf(1 - alpha/2)` times `std_dev/sqrt(size)`).
89 baseline lines in `eval/baseline/ironcalc-calc.txt` (48 NumericClose, 35 NumericUlp, 6 NumericMismatch):
- `ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx` `CONFIDENCE.NORM CONFIDENCE.T`: E2(C), E3, E5(C), E8(M), E11(C), E13(C), E16(C), E17(C), E22(C), E30(C), E34(C), E35(C), E36(C), E37(C), E48(C), E49(C), E53(C), E54(C), E56(C)
- `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST T.INV T.DIST.RT`: E2(C), E3, E4, E5, E6(C), E7(C), E8(C), E10(C), E11, E12, E13(C), E14, E15, E16(C), E17(C), E19(C), E20(M), E21(M), E22, E23, E28, E36, E38, E39, E40, E41, E42, E43, E46, E47, E55, E57, E58, E75(C), E76(C)
- `ironcalc-calc/STATISTICAL/T_DIST.xlsx` `T.DIST.2T T.INV.2T`: D2(C), D3(C), D4, D6, D7(C), D8(C), D11, D13, D16(C), D17(C), D18(C), D20(M), D25, D26, D34, D35, D38, D39, D47(C), D48(C), D49(C), D51(C), D52(C), D53(C), D54(C), D55(C), D56(C), D57(C), D58(C), D59(C), D60(C), D62(M), D63(M), D72, D73
(C = NumericClose, M = NumericMismatch, unmarked = NumericUlp.)

Examples (expected vs actual):
- `T.INV.2T(0.9999999,10)` D62: expected 1.2849890167888884e-07, actual 1.2467205919833128e-07 (3% off:
  cancellation near p = 0.5 in the inv_beta_reg route). D63 (p = 0.9999) 1.4e-8 relative.
- `T.INV(T.DIST(1E-6,11),11)` E20: expected 9.999999999666944e-07, actual 1.0000999564468016e-06.
- `CONFIDENCE.T(0.05,1,50)` E2 (9 lines with this value, plus scaled copies E13, E16, E17): expected 0.2841968554957298, actual 0.28419685549566487
  (2.3e-13). E8 (Mismatch): expected 1.781519460717496e-07, actual 1.7824180786974336e-07.
- `T.INV(T.DIST(1,1,TRUE),1)` E15: expected 1.0, actual 0.9999999999999996; `T.INV(...)` E11 expected
  1.0000000000000004, actual 1.0000000000000009 (Excel's own last digits; round trips are not exact,
  see `docs/excel-semantics/statistics.md`, "Inverse distribution functions do not round-trip exactly").

Hypothesis (the oracle is the judge): Excel solves CDF(t) = p iteratively on its own T.DIST kernel
(T-0969), so parity needs that kernel first; for the 2-tailed and near-0.5 cases solve on the tail
directly (no `1 - p`). The last digits depend on the solver's start, step and stopping rule:
**Needs an oracle sweep (T-0018):** fit against the T-1043 Student-t sweep, never blind.
Previous evidence from T-0925 (inverse normal): a closed-form inverse plus one Newton step regressed
cells that the old code matched by luck; measure regressions on every attempt.

Split (gap triage 2026-10-25): the centre quantiles, where a two-tailed p near 1 (or a one-tailed p near 0.5) makes
the result a tiny t and ours cancels (`TINV(1-1E-10,5)` Excel 1.3171528710518458e-10, ours 2.356080457693621e-08), are
owned by T-2724 (accuracy stage, 62 cells listed there, including T_DIST.xlsx D20, D62, D63 and E20 below). This task
keeps every other cell; its acceptance counts exclude the T-2724 cells.

Oracle sweep (T-1043): probes-numeric cases `m9-t-inv`, `m9-t-inv-2t`, `m9-confidence-t`, `m9-t-nonint-df`, `m9-t-edges`; findings in docs/excel-semantics/statistics.md "Student t sweep".

gen-m9 sweep (gap triage 2026-10-23, T-2255). CONFIDENCE.T, whose Student-t quantile is visibly off (35 cells).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 35 known-failure lines in eval/baseline/gen-m9.txt, 35 formulas.
Examples:
- gen-m9/146b46e74f2dac3b Calc!A340 `=CONFIDENCE.T(0.999999,1E-10,((2*(-2.5))+BITXOR(882,42)))`: Excel 4.2975693350469395e-18, ours 3.1591558575874862e-18
Cells:
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A340
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A389
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A45
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A181
- gen-m9/4bad303d32cc3f46.case.json (3): Calc!A5, A188, A371
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A57
- gen-m9/623663cacea53bbc.case.json (1): Calc!A293
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A53
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A372
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A256
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A246
- gen-m9/984cba67bbe6972b.case.json (5): Calc!A128, A177, A310, A317, A338
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A76, A320
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A4
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A132, A254
- gen-m9/b4173fe25109a238.case.json (1): Calc!A264
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A135
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A341
- gen-m9/c6cfda6bd10cae93.case.json (2): Calc!A99, A221
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A205
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A90
- gen-m9/ecc87f80077959e0.case.json (4): Calc!A5, A166, A327, A348
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A11

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: CONFIDENCE.T last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 5 known-failure lines in eval/baseline/gen-m9.txt, 5 formulas.
Examples:
- gen-m9/8fec7b207f4dd753 Calc!A185 `=CONFIDENCE.T(1E-16,1E+5,12)`: Excel 2388816.5175161078, ours 2388816.5175161064
Cells:
- gen-m9/8fec7b207f4dd753.case.json (2): Calc!A2, A185
- gen-m9/984cba67bbe6972b.case.json (2): Calc!A156, A191
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A198

T-2256 sweep (gen-m9 part 2, statistical M to Z; gap triage 2026-10-23): TINV, T.INV and T.INV.2T quantiles off Excel's: the central region (`TINV(0.5,9)` Excel 0.7027221467513249, ours 0.7027221467513268), tiny p (`TINV(1E-300,1)` 6.366197723675814E+299 against ...5218E+299), p near 1 where ours loses the small quantile to cancellation (`TINV(1-1E-10,10)` Excel 1.2849891237857122E-10, ours 3.332000937312528E-08; `TINV(1-1E-16,5)` 1.46E-16 against 0) and huge df (`TINV(1E-200,1E+10)` 58.19256428927105 against 30.2285..., T-1542's rule). p in (1,2) and p = 0 are T-2096's.

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 234 known-failure lines in eval/baseline/gen-m9.txt, 133 visibly off.
Examples:
- gen-m9/0620505280d2ea6e Calc!A28 `=TINV(0.5,9)`: Excel 0.7027221467513249, ours 0.7027221467513268
- gen-m9/46ac3ef5e61583ca Calc!A63 `=T.INV.2T(0.5,5)`: Excel 0.7266868438004216, ours 0.7266868438004208
- gen-m9/46ac3ef5e61583ca Calc!A187 `=T.INV.2T(0.5,3)`: Excel 0.7648923284043451, ours 0.7648923284043471
- gen-m9/5389dd4d5b5739d2 Calc!A299 `=TINV(1-1E-10,1)`: Excel 1.570797438519081e-10, ours 1.0536712127723509e-08
- gen-m9/775d053de498b8c7 Calc!A114 `=T.INV(0.3,4)`: Excel -0.5686490630497053, ours -0.5686490630497038
Cells:
- gen-m9/0072265d385d8ab2.case.json (3): Calc!A217, A242, A304
- gen-m9/0620505280d2ea6e.case.json (5): Calc!A28, A72, A134, A152, A276
- gen-m9/097af1ca33638670.case.json (6): Calc!A29, A101, A113, A175, A287, A361
- gen-m9/09b47629f54b9817.case.json (3): Calc!A40, A78, A268
- gen-m9/0bbab6206b5c9a07.case.json (6): Calc!A67, A93, A129, A315, A341, A358
- gen-m9/11eb3ab4f4f8104c.case.json (3): Calc!A32, A52, A176
- gen-m9/139684e68fb2fc3b.case.json (4): Calc!A52, A90, A204, A280
- gen-m9/17c110a53924beb8.case.json (3): Calc!A51, A175, A361
- gen-m9/1b31092a0d79e5f1.case.json (4): Calc!A31, A275, A279, A341
- gen-m9/1db18890916ee643.case.json (1): Calc!A144
- gen-m9/201d67f1d59436ed.case.json (4): Calc!A21, A207, A289, A295
- gen-m9/2138b909d8f810ea.case.json (3): Calc!A13, A89, A241
- gen-m9/329b177b5d5165ad.case.json (4): Calc!A62, A63, A203, A389
- gen-m9/334f3dc9ca937f10.case.json (5): Calc!A23, A128, A271, A293, A333
- gen-m9/38e4fc309fe199ea.case.json (4): Calc!A146, A260, A336, A374
- gen-m9/3b7990bd4e3307ba.case.json (3): Calc!A5, A58, A368
- gen-m9/3fb9cecf2c23169f.case.json (1): Calc!A183
- gen-m9/41248d1cb153c66e.case.json (4): Calc!A113, A171, A331, A361
- gen-m9/43c6227e0ff368c7.case.json (4): Calc!A70, A100, A147, A286
- gen-m9/44a8841352dc48e9.case.json (3): Calc!A74, A150, A226
- gen-m9/46ac3ef5e61583ca.case.json (5): Calc!A63, A114, A125, A187, A293
- gen-m9/5389dd4d5b5739d2.case.json (8): Calc!A60, A102, A122, A137, A184, A299, A361, A370
- gen-m9/5395b8a427764a3f.case.json (3): Calc!A37, A394, A397
- gen-m9/583ca2baea0bb30a.case.json (5): Calc!A147, A180, A242, A271, A332
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A53
- gen-m9/5be29fa316931f05.case.json (5): Calc!A3, A42, A104, A243, A305
- gen-m9/5cebac1f02bf30f2.case.json (1): Calc!A364
- gen-m9/607e85cfee83a689.case.json (2): Calc!A215, A353
- gen-m9/6e18550e9fc9c1b9.case.json (2): Calc!A73, A251
- gen-m9/775d053de498b8c7.case.json (5): Calc!A114, A176, A238, A326, A388
- gen-m9/787ba8d801105a60.case.json (5): Calc!A56, A79, A118, A148, A272
- gen-m9/7980ffb6095a340d.case.json (6): Calc!A46, A84, A122, A160, A236, A312
- gen-m9/7f7ca726c3f68554.case.json (2): Calc!A356, A394
- gen-m9/7fd0a98183c6babf.case.json (7): Calc!A90, A166, A204, A242, A318, A356, A394
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A123
- gen-m9/91e167cd8d7eb652.case.json (6): Calc!A102, A207, A269, A350, A386, A393
- gen-m9/928d08e3e1eecb5f.case.json (7): Calc!A15, A23, A85, A147, A201, A226, A395
- gen-m9/92d4fa29e5510f1e.case.json (2): Calc!A67, A295
- gen-m9/941c409532603d32.case.json (6): Calc!A12, A119, A170, A260, A305, A384
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A255
- gen-m9/9bc247733a1ca9fa.case.json (6): Calc!A17, A141, A203, A245, A327, A389
- gen-m9/9e2225ead8a6a07b.case.json (5): Calc!A40, A161, A175, A350, A361
- gen-m9/a2262ea466fcc142.case.json (5): Calc!A19, A193, A230, A255, A317
- gen-m9/a4090567758520df.case.json (4): Calc!A182, A201, A306, A369
- gen-m9/a5342a7e24fd803e.case.json (4): Calc!A42, A118, A187, A308
- gen-m9/a63d1c7d6f96c56d.case.json (3): Calc!A112, A254, A298
- gen-m9/afc00c8b70e78baf.case.json (6): Calc!A2, A39, A64, A309, A312, A349
- gen-m9/b9a32ce9a1fd0d71.case.json (1): Calc!A58
- gen-m9/bcb1105c61a2649d.case.json (4): Calc!A129, A191, A217, A315
- gen-m9/c2684066d8df263e.case.json (3): Calc!A51, A175, A275
- gen-m9/c4635963fc926284.case.json (1): Calc!A266
- gen-m9/c7664e494e418395.case.json (3): Calc!A65, A103, A179
- gen-m9/cb61169f4c033d4b.case.json (8): Calc!A45, A71, A107, A231, A257, A304, A319, A366
- gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A57
- gen-m9/d208bbafc3bc1997.case.json (4): Calc!A97, A135, A211, A325
- gen-m9/d37313406b9f6e55.case.json (1): Calc!A57
- gen-m9/d5fd9e05e9e6f466.case.json (5): Calc!A22, A145, A146, A207, A331
- gen-m9/d8f44d556674c2fd.case.json (4): Calc!A88, A164, A240, A392
- gen-m9/d910f000b60e2eac.case.json (4): Calc!A26, A176, A212, A300
- gen-m9/eccd03df42e40006.case.json (2): Calc!A45, A273
- gen-m9/ede0323c389fc703.case.json (3): Calc!A110, A148, A262
- gen-m9/f82473275380882e.case.json (4): Calc!A291, A320, A353, A382

Split 2026-10-27 (gap triage, opus refill): while this task waits on T-0969, two parts that need no new CDF are
their own tasks. T-2858 (sonnet) owns every df = 1 cell: Excel's df = 1 quantile is `1/TAN(PI()*p/2)` through
Excel's TAN, bit-exact on all 112 failing df = 1 cells (statistics.md, T-2858 entry). T-2859 (opus, `accuracy`)
owns the visibly-off cells for df 2 to 99998 with p/2 >= 2E-5 where Excel is within a few ulps of the true
quantile and ours is not (274 cells, the list is in T-2859). This task keeps the bit-level target, the cells
T-2859 shows Excel itself is off on, and whatever T-2858 and T-2859 leave in their Outcomes.

Gap triage 2026-10-27 (`make eval` on 7dba3bfe): 3 visibly-off cells that no task named. They are p in (1,2), which
T-2096 maps to `T.INV(1 - p/2, df)`, and the engine gives the same digits for `T.INV(0.25,df)`, `TINV(1.5,df)` and
`-T.INV(0.75,df)`. So the miss is the central quantile itself, not the p > 1 mapping. Exact (mpmath)
`T.INV(0.25,7)` is -0.71114177808178631: Excel is 4E-16 above it, ours 3E-16 below.
- gen-m9/5395b8a427764a3f Calc!A347 `=T.INV.2T(1.5,7)`: Excel -0.7111417780817859, ours -0.7111417780817866
- gen-m9/787ba8d801105a60 Calc!A265 `=TINV(1.5,7)`: Excel -0.7111417780817859, ours -0.7111417780817866
- gen-m9/1db18890916ee643 Calc!A106 `=TINV(1.5,10)`: Excel -0.6998120613124317, ours -0.6998120613124311

Sequencing (T-0098, 2026-10-28): waits on T-2511, which inverts the ported incomplete beta for T.INV; fit the
quantile on the ported kernel, not the current one, so the work is not done twice.

T-3061 (gap triage 2026-10-28) takes the cells away from the centre where Excel's quantile is SQRT(df*(1/y-1)) of the
rounded beta quantile, at df 1000 to 1E10 and wherever the one-sided q is at most 0.45 (statistics.md "Student-t
quantile is SQRT(df*(1/y-1)) of the rounded beta quantile"). It does not depend on T-0969. Rescore this task's
cells after it lands.
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - gen-m9/334f3dc9ca937f10.case.json: Calc!A252
  - gen-m9/91e167cd8d7eb652.case.json: Calc!A226
  - gen-m9/a2262ea466fcc142.case.json: Calc!A7
  - gen-m9/a63d1c7d6f96c56d.case.json: Calc!A370
  - gen-m9/5919660b7bb4d195.case.json: Calc!A395
  - gen-m9/5be29fa316931f05.case.json: Calc!A127
  - gen-m9/afc00c8b70e78baf.case.json: Calc!A188, A371
  - gen-m9/ede0323c389fc703.case.json: Calc!A186
  - gen-m9/1b31092a0d79e5f1.case.json: Calc!A57
  - gen-m9/c2684066d8df263e.case.json: Calc!A64
  - gen-m9/ee2497a5e683c5d9.case.json: Calc!A319

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255 and T-2256): these 269 known-failure lines (35 from T-2255, 234 from T-2256) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] All 6 NumericMismatch and all 48 NumericClose lines above leave the NumericClose/Mismatch
      classes, at least 45 of the 89 lines leave `eval/baseline/ironcalc-calc.txt`, no new lines in
      any baseline; remaining lines listed in the PR with expected/actual.
- [ ] One quantile routine shared by T.INV, T.INV.2T and CONFIDENCE.T (no remaining statrs
      `StudentsT::inverse_cdf` call; `rg inverse_cdf` shows it).
- [ ] Unit tests for p near 0, 0.5 and 1 and df = 1, 2, 1e6; bounded iteration (no unbounded loops
      on untrusted input).
- [ ] `make check` green; reviewers approve.

## Out of scope
The CDF kernel (T-0969); CONFIDENCE.NORM (T-0925); legacy TINV (evidence in T-1048).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of Excel's value (an expected 0 must be exactly 0). Criteria above that ask for bit-exact or ulp-level matches and are still unmet then move to a follow-up task (no campaign key), named in the Outcome. Regressions that stay within 4 ulp follow decision 0083's allowance.

Visibly-off cells on the 2026-10-20 eval run (`main` 35a5e71), blamed on this function rather than on glue around it: 345 in total.
- T.INV: 109 cells (13-14 digits right 66, 10-12 33, 6-9 4, fewer or 0 vs nonzero 6); most in `probes-numeric/m9-t-inv.case.json` 67; `probes-numeric/t1531-t-tiny-p.case.json` 27. e.g. `T.INV(0.4,5)` Excel -0.2671808657041464, ours -0.2671808657041447
- T.INV.2T: 148 cells (13-14 digits right 69, 10-12 45, 6-9 12, fewer or 0 vs nonzero 22); most in `probes-numeric/m9-t-inv-2t.case.json` 100; `ironcalc-calc/STATISTICAL/T_DIST.xlsx` 19. e.g. `T.INV.2T(0.5,2)` Excel 0.8164965809277259, ours 0.8164965809277231
- TINV: 22 cells (13-14 digits right 15, 10-12 4, 6-9 1, fewer or 0 vs nonzero 2); most in `probes-numeric/t1531-t-tiny-p.case.json` 17; `probes-numeric/t1048-tinv.case.json` 5. e.g. `TINV(0.5,10)` Excel 0.6998120613124317, ours 0.6998120613124311
- CONFIDENCE.T: 66 cells (13-14 digits right 31, 10-12 29, 6-9 3, fewer or 0 vs nonzero 3); most in `probes-numeric/m9-confidence-t.case.json` 30; `ironcalc-calc/STATISTICAL/CONFIDENCE.xlsx` 18. e.g. `CONFIDENCE.T(0.1,1,2)` Excel 4.464496510753559, ours 4.464496510753552

Against a 50-digit reference, Excel is close to correctly rounded for this family and the engine is the inaccurate side, so an accurate kernel (not a fit) should reach this stage.

## Cells added by the Phase 1 triage (T-0090, 2026-10-27)
From the parked T-2695: CONFIDENCE.T gen-m9/85f48d5cdfb24555 Calc!A372 `CONFIDENCE.T(0.3,0.3,29)`, 0.0588276537815129 expected, 0.05882765378149652 today.

## Cells back from T-2724 (2026-10-31)
T-2724's centre kernel (an accurate quantile, never a fit) leaves these owned cells within about 2E-16 of the true
quantile (mpmath) but more than half a unit of the 15th digit from Excel, whose own last digits are 1E-15 to 3.6E-15
relative off there (statistics.md "Student t sweep", "Centre quantiles are accurate"): gen-m9/09b47629f54b9817 A40
`TINV(1-1E-16,8)`, 4bad303d32cc3f46 A5 and A371, 984cba67bbe6972b A128, ecc87f80077959e0 A327 `CONFIDENCE.T(1-1E-10,2.5,16)`.
