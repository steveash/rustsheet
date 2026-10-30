---
id: T-3233
title: "F and beta quantile last digits on the m9, m9-legacy and t1049 probes: F.INV, F.INV.RT, FINV, BETA.INV, BETAINV (227 cells within 15 digits)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2500, T-2511]
defer: phase-1b
---

## Goal
F.INV, F.INV.RT, FINV, BETA.INV and BETAINV match Excel to the bit on the 227 quantile cells listed below. Each
cell is already within half a unit of Excel's 15th significant digit, and no open task lists any of them.

## Context
- Gap triage 2026-10-30 (`make eval` at e1255db9, excel-graph evidence). Each cell has one of these quantiles as its
  only non-glue call. By function: F.INV.RT 87, F.INV 65, FINV 53, BETAINV 12, BETA.INV 10. By suite: probes-numeric
  146, gen-m9 77, ironcalc-calc 2, probes 2.
- Ulp distance, ours against Excel's: 1 ulp 61, 2 ulp 49, 3 ulp 42, 4 ulp 31, 5 to 9 ulp 37, 10 to 30 ulp 7. The
  verdicts are 183 `NumericUlp` and 44 `NumericClose`. The report marks none of them visibly off; the closest call is
  probes-numeric/t1049-finvrt Calc!A2, `F.INV.RT(0.01,3,7)`, Excel 8.45128505307999 against ours 8.451285053079985.
- Neighbouring tasks, none of which lists these cells:
  - T-2511 re-bases every F and beta quantile on the ported incomplete beta's crossing.
  - T-2500 owns the F quantile cells a unit or two off the 15th digit.
  - T-2767 (deferred) owns a separate set of 28 gen-m9 F quantile cells. This task is its sibling for the
    probe sweeps and the gen-m9 cells generated since.
  - T-2808 owns the far-tail saturation and BETAINV's 0.5 quirk.
  - T-2555 and T-2936 own BETA.INV at a tiny quantile or a huge shape.
  T-2511 and T-2500 move the crossing these cells sit on, so re-run this list after them before any fit.
- Examples (`make eval-show CASE=<case id>` for the cases in the list below), Excel against ours:
  - probes-numeric/m9-f-inv-rt Calc!A8 `=F.INV.RT(1e-10,2,5)`: 24997.499999999975 against 24997.5. At df1 = 2 the
    exact quantile is (d2/2)(p^(-2/d2) - 1) = 2.5·(10^4 - 1) = 24997.5, so Excel does not use this closed form
    (Excel is always right). This quirk is not yet in docs/excel-semantics/statistics.md: record it there.
  - probes-numeric/m9-legacy-finv Calc!A17 `=FINV(0.01,1,1)` and A18 `=F.INV.RT(0.01,1,1)`: both 4052.1806954768263
    against 4052.180695476829 (6 ulp), so FINV and F.INV.RT share the path.
  - probes-numeric/m9-legacy-betainv Calc!A3 `=BETAINV(1e-06,0.5,0.5)`: 2.4674011002703103e-12 against
    2.4674011002703067e-12 (9 ulp).
  - gen-m9/3a25a03eb94fcfbd.case.json Calc!A327 `=F.INV.RT(1-1E-16,2,8)`: 1.1102230246251531e-16 against
    1.1102230246251494e-16 (30 ulp, still within 15 digits).
  - ironcalc-calc/STATISTICAL/F_DIST.xlsx Sheet1!J2 and L2, F.INV and F.INV.RT round trips that come back to
    about 0.8: Excel 0.8000000000000002 against ours 0.7999999999999998. Excel's round trip lands above x, ours below it.
- Suspected root cause: Excel's x is a crossing of its own cdf (T-1745: statistics.md "BETA.INV solves Excel's own CDF
  to the crossing"; T-0974: "F.INV is a crossing of F.DIST's tails"). The cdf these searches cross is a few ulp from
  Excel's, or Excel's search stops at a different iterate. The search is `f_quantile` / `f_quantile_tail` in
  crates/rustsheet-engine/src/functions/statistical/fisher.rs and `beta_quantile_crossing` in inverse_beta.rs.
  Numeric fitting, so opus.
- Evidence: the cached cases in the list below give 227 failing points and several hundred passing ones. If they
  still cannot separate the candidate stopping rules, do not fit blind. First file a research-lane oracle sweep of
  F.INV.RT and BETA.INV over a dense p grid at the listed shapes, and depend on it.

### Cells owned
Each cell below has F.INV, F.INV.RT, FINV, BETA.INV or BETAINV as its only non-glue call:
- probes-numeric/m9-legacy-finv.case.json (44): Calc!A5, A6, A7, A8, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A22, A23, A24, A25, A26, A29, A30, A31, A32, A33, A34, A35, A36, A41, A42, A43, A44, A45, A46, A49, A50, A51, A52, A53, A54, A55, A56, A69, A72
- probes-numeric/m9-f-inv-rt.case.json (29): Calc!A2, A3, A4, A5, A6, A8, A9, A11, A12, A13, A14, A16, A17, A18, A20, A21, A22, A23, A24, A26, A27, A28, A29, A30, A31, A32, A33, A34, A35
- probes-numeric/m9-f-inv.case.json (27): Calc!A2, A3, A5, A6, A8, A9, A11, A12, A13, A14, A15, A16, A17, A19, A20, A22, A23, A24, A26, A27, A28, A29, A31, A32, A33, A34, A35
- probes-numeric/m9-legacy-betainv.case.json (20): Calc!A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16, A17, A18, A21, A22, A25, A26
- probes-numeric/t1049-finv.case.json (13): Calc!A1, A2, A3, A4, A5, A6, A8, A9, A11, A13, A15, A22, A23
- probes-numeric/t1049-finvrt.case.json (13): Calc!A1, A2, A5, A7, A8, A9, A10, A11, A12, A13, A15, A22, A23
- gen-m9/27601819397dd56e.case.json (4): Calc!A10, A148, A376, A378
- gen-m9/530c636749a85209.case.json (4): Calc!A63, A175, A316, A339
- gen-m9/589f42e67f4b4b8d.case.json (4): Calc!A74, A231, A350, A396
- gen-m9/899f9555e67da2d6.case.json (4): Calc!A49, A184, A230, A249
- gen-m9/4e55500bb8ad3cd8.case.json (3): Calc!A148, A155, A332
- gen-m9/c18dbf21f11093f3.case.json (3): Calc!A184, A213, A400
- gen-m9/ee2497a5e683c5d9.case.json (3): Calc!A12, A180, A318
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A72, A355
- gen-m9/0a9226016e8f5f33.case.json (2): Calc!A5, A183
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A10, A321
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A56, A369
- gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A327, A388
- gen-m9/40425341c5034eed.case.json (2): Calc!A45, A246
- gen-m9/58e3253bbd3a7a82.case.json (2): Calc!A80, A272
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A28, A258
- gen-m9/68d8ffb69ff739d8.case.json (2): Calc!A1, A319
- gen-m9/7674c89c824aca43.case.json (2): Calc!A166, A375
- gen-m9/7c20351562a2d71d.case.json (2): Calc!A221, A392
- gen-m9/a54c738c697617c3.case.json (2): Calc!A216, A311
- gen-m9/a5828d3925575713.case.json (2): Calc!A69, A164
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A191, A367
- gen-m9/cde551ea787f713a.case.json (2): Calc!A123, A306
- gen-m9/d8266bc15533702a.case.json (2): Calc!A272, A394
- ironcalc-calc/STATISTICAL/F_DIST.xlsx (2): Sheet1!J2, L2
- gen-m9/06b4e8f5de409d5e.case.json (1): Calc!A145
- gen-m9/165db58d1540f014.case.json (1): Calc!A150
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A241
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A199
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A222
- gen-m9/4250f880ab640c49.case.json (1): Calc!A60
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A18
- gen-m9/623663cacea53bbc.case.json (1): Calc!A70
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A341
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A75
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A272
- gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A337
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A67
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A169
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A198
- gen-m9/c468f5046aadfbbc.case.json (1): Calc!A241
- gen-m9/e6cf3b20a014e6a3.case.json (1): Calc!A351
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A295
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A56
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A249
- probes/t2436/05d44f6dd6cfc567.case.json (1): Calc!A46
- probes/t2436/a612e89321a6df9c.case.json (1): Calc!A76

## Acceptance criteria
- [ ] These known-failure lines disappear from the baselines, with no new ones in any baseline (`make eval-baseline`,
      `make eval-check`: 0 regressions):
      - the 77 gen-m9 lines from eval/baseline/gen-m9.txt;
      - the 2 ironcalc-calc lines from eval/baseline/ironcalc-calc.txt;
      - the 2 probes lines from eval/baseline/probes.txt.
- [ ] probes-numeric is ungated, so its 146 owned cells are scored from `make eval`'s report.json: they pass
      bit-exact. Or, under the decision 0056 time box, the Outcome records the models tried with their bit-exact
      counts, and the open cells move to a follow-up.
- [ ] docs/excel-semantics/statistics.md has an entry for the df1 = 2 F.INV.RT value above that is not the closed
      form, citing probes-numeric/m9-f-inv-rt Calc!A8, and records the stopping rule the fit finds.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off F and beta quantile cells (T-2500, T-2633, T-1871) and the port itself (T-2511).
- T.INV, T.INV.2T and TINV (T-0967, T-3061).
