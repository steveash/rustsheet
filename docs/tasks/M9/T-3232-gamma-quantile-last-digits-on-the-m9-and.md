---
id: T-3232
title: "Gamma quantile last digits on the m9 and t1048/t1049 probes and GAMMA.xlsx: GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT, CHIINV (293 cells within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3016, T-3131, T-3146]
defer: phase-1b
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV match Excel to the bit on the 293 quantile cells listed below.
Each cell is already within half a unit of Excel's 15th significant digit, and no open task lists any of them.

## Context
- Gap triage 2026-10-30 (`make eval` at e1255db9, excel-graph evidence). Each cell has a gamma quantile as its only
  non-glue call. The 293 cells: GAMMA.INV 89, CHISQ.INV.RT 81, CHISQ.INV 81, CHIINV 29, GAMMAINV 13. By suite:
  probes-numeric 150, gen-m9 114, ironcalc-calc 25, probes 4.
- Ulp distance, ours against Excel's: 1 ulp 180, 2 ulp 44, 3 ulp 22, 4 ulp 19, 5 to 8 ulp 20, 10 ulp 8. The verdicts
  are 265 `NumericUlp` and 28 `NumericClose`, and none is visibly off.
- Neighbouring tasks, none of which lists these cells:
  - T-3016 owns the 49 gated cells T-0972 moved by 1-3 ulp, plus five visibly-off cells.
  - T-3131 owns T-0972's remaining visibly-off cells and the statrs search start (`inverse_cdf_within`).
  - T-3146 owns the p = 0.999 landing band.
  - T-2561 owns the deep tail.
  - T-2695 (deferred) owns a separate set of gen-m9 CHIINV, GAMMAINV and GAMMA.INV cells.
  - T-2367 (deferred) owns GAMMA.INV F38 and the inversion cap.
  All of the first three change the search these cells sit on, so re-run this list after them before fitting.
- Examples (`make eval-show CASE=<case id>` for the cases in the list below), Excel against ours:
  - probes-numeric/m9-gamma-inv Calc!A1 `=GAMMA.INV(1e-10,0.5,1)`: 7.853981633974484e-21 against 7.853981633974482e-21.
  - probes-numeric/m9-gamma-inv Calc!A5 `=GAMMA.INV(0.95,0.5,1)`: 1.9207294103470618 against 1.920729410347064 (10 ulp).
  - probes-numeric/m9-chisq-inv Calc!A5 `=CHISQ.INV(0.95,1)`: 3.8414588206941236 against 3.841458820694128 (10 ulp).
    A 50-digit decimal bisection on erf gives 3.841458820694126 for the correctly rounded value, so Excel is closer
    to the exact quantile than ours here.
  - probes-numeric/m9-chisq-inv Calc!A4 `=CHISQ.INV(0.5,1)`: 0.4549364231195729 against 0.4549364231195725 (7 ulp).
    The exact value is 0.4549364231195728, so Excel is again the closer one.
  - ironcalc-calc/STATISTICAL/GAMMA.xlsx `GAMMA.DIST GAMMA.INV!F7`, a GAMMA.INV of the sheet's own GAMMA.DIST at
    x = 2: 1.9999999999999996 against 2.0. Excel stops one ulp short on the round trip.
- Suspected root cause: two parts.
  - At shape 0.5 (df 1) the engine's refinement is less accurate than Excel, up to 10 ulp from the exact root where
    Excel is 1-2 ulp from it. That part is a more accurate crossing (accuracy work, not a fit).
  - The rest is Excel's own stop: Excel's x is a point where its own cdf crosses p, and Excel's search stops at an
    iterate the engine does not reproduce (the round-trip F7 above).
  The quantile code is `gamma_inverse_cdf` / `inverse_cdf_within` and T-0972's crossing refinement in
  crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs, called from gamma.rs and chisq.rs.
  Numeric fitting, so opus.
- Evidence: the cached cases in the list below give 293 failing points and several hundred passing ones. If they
  still cannot separate the candidate stopping rules, do not fit blind. First file a research-lane oracle sweep of
  GAMMA.INV over a dense p grid at shapes 0.5, 1, 3 and 9, and depend on it.

### Cells owned
Each cell below has GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT or CHIINV as its only non-glue call:
- probes-numeric/m9-gamma-inv.case.json (55): Calc!A1, A4, A5, A6, A7, A9, A10, A11, A12, A13, A14, A15, A16, A18, A19, A20, A23, A24, A26, A27, A30, A33, A36, A40, A42, A43, A44, A46, A47, A48, A49, A50, A54, A55, A56, A58, A61, A62, A63, A65, A66, A67, A68, A69, A70, A72, A74, A75, A76, A79, A80, A81, A82, A83, A84
- probes-numeric/m9-chisq-inv-rt.case.json (30): Calc!A1, A2, A3, A4, A5, A6, A7, A8, A10, A11, A13, A16, A17, A18, A19, A21, A23, A24, A25, A26, A28, A29, A30, A31, A32, A33, A42, A43, A45, A47
- probes-numeric/m9-chisq-inv.case.json (28): Calc!A1, A4, A5, A6, A7, A8, A9, A11, A12, A13, A15, A16, A18, A21, A22, A23, A25, A28, A29, A30, A32, A33, A37, A42, A44, A47, A48, A49
- ironcalc-calc/STATISTICAL/GAMMA.xlsx (24): GAMMA.DIST GAMMA.INV!F2, F7, F10, F19, F22, F24, F39, F41, F43, F44, F45, F46, F47, F50, F51, F53, F61, F62, F67, F68, F69, F70, F74, F76
- probes-numeric/t1049-chisqinvrt.case.json (16): Calc!A1, A3, A4, A5, A6, A7, A8, A9, A10, A11, A13, A15, A17, A18, A26, A28
- probes-numeric/t1049-chisqinv.case.json (13): Calc!A2, A3, A4, A9, A10, A11, A13, A14, A17, A18, A20, A26, A28
- gen-m9/984cba67bbe6972b.case.json (8): Calc!A41, A85, A90, A104, A118, A132, A209, A216
- probes-numeric/t1048-gammainv.case.json (8): Calc!A2, A3, A4, A8, B2, B3, B4, B8
- gen-m9/285d87cea1aaa73c.case.json (5): Calc!A8, A132, A139, A200, A252
- gen-m9/40425341c5034eed.case.json (4): Calc!A17, A247, A351, A385
- gen-m9/58e3253bbd3a7a82.case.json (4): Calc!A8, A37, A267, A287
- gen-m9/8d3588d8df0812df.case.json (4): Calc!A116, A232, A289, A293
- gen-m9/ecc87f80077959e0.case.json (4): Calc!A99, A133, A224, A322
- gen-m9/05d44f6dd6cfc567.case.json (3): Calc!A16, A215, A315
- gen-m9/0a614cfe974b59c3.case.json (3): Calc!A32, A276, A327
- gen-m9/27601819397dd56e.case.json (3): Calc!A47, A310, A356
- gen-m9/3e98b26cf6713e77.case.json (3): Calc!A21, A175, A326
- gen-m9/8f2ac25945af87da.case.json (3): Calc!A25, A64, A301
- gen-m9/a612e89321a6df9c.case.json (3): Calc!A81, A261, A307
- gen-m9/a7bf0d8d81d33ba6.case.json (3): Calc!A129, A216, A382
- gen-m9/c2974c061384fdd9.case.json (3): Calc!A58, A119, A297
- gen-m9/cde551ea787f713a.case.json (3): Calc!A45, A209, A256
- gen-m9/d8266bc15533702a.case.json (3): Calc!A247, A248, A334
- gen-m9/ee2497a5e683c5d9.case.json (3): Calc!A56, A189, A378
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A100, A146, A201
- probes/incomplete-gamma-cap.case.json (3): Sheet1!E5, E14, E19
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A61, A311
- gen-m9/165db58d1540f014.case.json (2): Calc!A53, A209
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A283, A332
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A209, A349
- gen-m9/530c636749a85209.case.json (2): Calc!A228, A365
- gen-m9/7674c89c824aca43.case.json (2): Calc!A152, A290
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A235, A327
- gen-m9/85f48d5cdfb24555.case.json (2): Calc!A22, A382
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A4, A36
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A20, A348
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A61, A164
- gen-m9/bc40d37cb4d8301d.case.json (2): Calc!A163, A371
- gen-m9/bcb24385baafb2f0.case.json (2): Calc!A81, A386
- gen-m9/f34f66fb7f81e673.case.json (2): Calc!A102, A172
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A132
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A378
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A372
- gen-m9/4250f880ab640c49.case.json (1): Calc!A66
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A39
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A79
- gen-m9/589f42e67f4b4b8d.case.json (1): Calc!A34
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A15
- gen-m9/623663cacea53bbc.case.json (1): Calc!A238
- gen-m9/899f9555e67da2d6.case.json (1): Calc!A228
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A40
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A143
- gen-m9/a013a45a8fbcab4f.case.json (1): Calc!A59
- gen-m9/a54c738c697617c3.case.json (1): Calc!A310
- gen-m9/a5828d3925575713.case.json (1): Calc!A142
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A141
- gen-m9/b4173fe25109a238.case.json (1): Calc!A69
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A119
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A6
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A282
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A61
- ironcalc-calc/STATISTICAL/CHISQ.xlsx (1): Sheet1!I2
- probes/t2436/8d3588d8df0812df.case.json (1): Calc!A1

## Acceptance criteria
- [ ] These known-failure lines disappear from the baselines, with no new ones in any baseline (`make eval-baseline`,
      `make eval-check`: 0 regressions):
      - the 114 gen-m9 lines from eval/baseline/gen-m9.txt;
      - the 25 ironcalc-calc lines from eval/baseline/ironcalc-calc.txt;
      - the 4 probes lines from eval/baseline/probes.txt.
- [ ] probes-numeric is ungated, so its 150 owned cells are scored from `make eval`'s report.json: they pass
      bit-exact. Or, under the decision 0056 time box, the Outcome records the models tried with their bit-exact
      counts, and the open cells move to a follow-up.
- [ ] docs/excel-semantics/statistics.md records the gamma quantile's search and stopping rule, citing these cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off gamma quantile cells (T-3016, T-3131, T-3146, T-2561, T-3070) and the huge-shape #NUM! band (T-2982).
- The gen-m9 cells T-2695 lists, and the incomplete gamma cdf itself (T-3234, T-2367).
