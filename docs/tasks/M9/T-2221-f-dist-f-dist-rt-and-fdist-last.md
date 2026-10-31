---
id: T-2221
title: "F.DIST, F.DIST.RT and FDIST last digits: fit Excel's closed forms, complement and tail choice on the sweep"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2219, T-2220]
defer: phase-1b
---

## Goal
F.DIST (cdf and density), F.DIST.RT and FDIST give Excel's double bit for bit on the F-distribution
probes, including where Excel is not correctly rounded. Needs an oracle sweep: T-2220 provides it (`probes-numeric/t2220-f-dist-sweep-*.case.json`, findings in statistics.md "Dense F sweep (T-2220)").

## Context
Gap triage 2026-10-21 (`make eval` on `main` fce6020d). After T-0970, 107 F-distribution cells
are still not bit-exact and no open task owns them. They exclude T-2219's 16 accuracy cells and the
cells owned by T-1849 (booleans), T-2095 (x = 0) and T-1538 (df of 1E7 and above):
- ironcalc-calc/STATISTICAL/F_DIST.xlsx Sheet1!F3 (`F.DIST(0.7,4,2.4,FALSE)`, Excel
  0.40509259259259256, ours 1 ulp above). This is a known-failure line in eval/baseline/ironcalc-calc.txt.
- probes-numeric/m9-f-dist-rt A1-A4, A13-A16, A19, A20.
- probes-numeric/m9-f-dist A1-A6, A8, A10, A16, A18, A19, A21, A22, A24.
- probes-numeric/m9-legacy-fdist A21, A22, A25, A26, A29-A32, A35, A36, A41, A42, A45-A52,
  A55-A62, A65-A72, A75-A80.
- probes-numeric/t1049-fdist A1-A3, A5, A7-A10, A12-A14, A16-A18, A20, A22-A24, A32-A35, A37, A38, A43.
- probes-numeric/t1049-fdistrt A1, A2, A4, A5, A7-A12, A17, A18, A23-A25.

Most are 1-4 ulp. Six are visibly off (more than half a unit in the 15th digit) because Excel itself is
off the true value by more than that, so no accurate kernel reaches them. A 50-digit mpmath
reference, run in the 2026-10-21 triage, gives these relative errors:
- m9-f-dist-rt A16 and m9-legacy-fdist A55, A56: `F.DIST.RT(10,10,10)` = 0.0005715525434020334,
  Excel +1.4E-15, ours -5.4E-16.
- m9-f-dist-rt A20: `F.DIST.RT(10,50,100)` = 3.009600356083383E-22, Excel -4.1E-15, ours -2.3E-15.
- m9-legacy-fdist A47, A48: `FDIST(3.5,5,50)` = 0.008642212995925193, Excel +7.1E-16, ours -1.7E-15.
- t1049-fdist A16: `F.DIST(2.5,3,7,FALSE)` = 0.09478619896436714, Excel +9.5E-16, ours -2.2E-16.

Excel's d1 = d2 = 1 values are not the exact ones either: `F.DIST(1,1,1,TRUE)` = 0.5000000000000001,
`F.DIST.RT(1,1,1)` = 0.4999999999999999, and `F.DIST(0.5,1,1,TRUE)` = 0.39182655203060723, 1 ulp below
`2/PI()*ATAN(SQRT(0.5))` in doubles. statistics.md's T-0970 section has the arctangent lead and the
two PARTIAL rules (exact complement, one-evaluation tails) that the oracle contradicts at
t1049-fdistrt A2, A8. Fit lab precedent: T-1744 (`uv run rustsheet-fitlab`) and
docs/research/excel-kernel-playbook.md. Code: `functions/statistical/incomplete_beta.rs` and the F
callers in `functions/statistical/`.

gen-m9 sweep (T-2255 triage), last digits only, not visibly off: FDIST, F.DIST.RT and F.DIST last digits. Bit-level targets, outside the visible-parity stage (T-2255 triage, excel-graph, 2026-10-22): 137 known-failure lines in eval/baseline/gen-m9.txt, 137 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A114 `=FDIST(2,3,1)`: Excel 0.4695222290670432, ours 0.4695222290670429
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A114
- gen-m9/06b4e8f5de409d5e.case.json (3): Calc!A91, A111, A213
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A202, A230
- gen-m9/146b46e74f2dac3b.case.json (2): Calc!A169, A291
- gen-m9/165db58d1540f014.case.json (1): Calc!A238
- gen-m9/2801c1c11c21e134.case.json (4): Calc!A69, A184, A191, A374
- gen-m9/285d87cea1aaa73c.case.json (3): Calc!A50, A233, A355
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A106
- gen-m9/35e8168aae2b00f8.case.json (4): Calc!A104, A287, A348, A367
- gen-m9/3a25a03eb94fcfbd.case.json (3): Calc!A59, A171, A354
- gen-m9/3e98b26cf6713e77.case.json (2): Calc!A28, A272
- gen-m9/40425341c5034eed.case.json (2): Calc!A111, A203
- gen-m9/4250f880ab640c49.case.json (4): Calc!A12, A157, A162, A223
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A111
- gen-m9/4bad303d32cc3f46.case.json (5): Calc!A20, A80, A263, A272, A324
- gen-m9/4e55500bb8ad3cd8.case.json (3): Calc!A89, A273, A319
- gen-m9/530c636749a85209.case.json (2): Calc!A202, A248
- gen-m9/58e3253bbd3a7a82.case.json (2): Calc!A135, A273
- gen-m9/60b0b6e69ad35a45.case.json (3): Calc!A151, A326, A334
- gen-m9/623663cacea53bbc.case.json (3): Calc!A120, A121, A304
- gen-m9/68d8ffb69ff739d8.case.json (4): Calc!A90, A110, A125, A247
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A395
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A43, A89
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A103, A164
- gen-m9/8dd2312c96effb1b.case.json (3): Calc!A239, A300, A388
- gen-m9/8f2ac25945af87da.case.json (4): Calc!A83, A266, A336, A358
- gen-m9/8fec7b207f4dd753.case.json (3): Calc!A249, A309, A370
- gen-m9/9cca17438a7e6f8a.case.json (4): Calc!A50, A164, A286, A294
- gen-m9/a013a45a8fbcab4f.case.json (5): Calc!A71, A179, A252, A254, A376
- gen-m9/a54c738c697617c3.case.json (4): Calc!A18, A117, A176, A262
- gen-m9/a5828d3925575713.case.json (1): Calc!A51
- gen-m9/a612e89321a6df9c.case.json (3): Calc!A116, A162, A392
- gen-m9/a61e547a1cab994c.case.json (2): Calc!A167, A279
- gen-m9/a790a4d4a6a82756.case.json (2): Calc!A60, A243
- gen-m9/a7bf0d8d81d33ba6.case.json (4): Calc!A108, A291, A311, A352
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A42, A318
- gen-m9/b4173fe25109a238.case.json (4): Calc!A180, A236, A363, A380
- gen-m9/b8ee55934c5b824c.case.json (4): Calc!A160, A206, A252, A344
- gen-m9/bbdfc06cae0e0d71.case.json (2): Calc!A35, A81
- gen-m9/bc40d37cb4d8301d.case.json (2): Calc!A9, A375
- gen-m9/bcb24385baafb2f0.case.json (3): Calc!A1, A123, A184
- gen-m9/c18dbf21f11093f3.case.json (2): Calc!A59, A105
- gen-m9/c2974c061384fdd9.case.json (3): Calc!A27, A147, A165
- gen-m9/c468f5046aadfbbc.case.json (3): Calc!A15, A98, A320
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A188
- gen-m9/cde551ea787f713a.case.json (3): Calc!A59, A149, A364
- gen-m9/d8266bc15533702a.case.json (2): Calc!A215, A259
- gen-m9/e6cf3b20a014e6a3.case.json (3): Calc!A111, A146, A207
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A49, A95, A187
- gen-m9/fd9fa22d212080d6.case.json (3): Calc!A6, A183, A366
- gen-m9/fec39efaba9284c3.case.json (2): Calc!A184, A322

T-2502 (2026-10-25) settles part of this: F.DIST, F.DIST.RT and FDIST at a df of 2 take Excel's closed forms on
bratio's oriented arguments with w5's x = 1/(1 + d2/(d1·X)), y = 1/(1 + (d1·X)/d2), bit-exact on all 367 F closed-form
cells of T-2464 w5-triangulation (t2220-f-dist-sweep-cdf, -rt, -d1-1, -d2-1: 70 cells fixed). The tail choice is
bratio's dispatch, not the mean (statistics.md "Incomplete beta is TOMS 708 `bratio`"); the other shapes' last digits
need bratio's branches (T-2505 onwards).

## Acceptance criteria
- [ ] The F_DIST.xlsx Sheet1!F3 line disappears from eval/baseline/ironcalc-calc.txt
      (`make eval-baseline`), with no new line in any baseline.
- [ ] Of the 106 probes-numeric cells listed above, at least 90% are bit-exact, and every one is within
      half a unit of the 15th significant digit. The ones that are not bit-exact are listed in the
      Outcome with their ulp distance.
- [ ] Each T-2220 sweep cell of F.DIST, F.DIST.RT and FDIST is bit-exact or within 4 ulp. The rate is
      reported in the Outcome.
- [ ] statistics.md's T-0970 section replaces the PARTIAL rules with the fitted formulation (VERIFIED on
      the sweep) or records which hypotheses were ruled out.
- [ ] The 35 cells added by gap triage 2026-10-30 are bit-exact (their 6 gen-m9 lines leave eval/baseline/gen-m9.txt; the probes-numeric cells, ungated, shown with `make eval-show`) or the Outcome lists each with its ulp distance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The accuracy fix at large a + b (T-2219, a dependency). F.INV and F.INV.RT (T-0974, which should
  invert whatever cdf this lands). BETA.DIST's cdf (T-2119). df of 1E7 and above (T-1538, T-1864).

## Visible-parity stage
Decision 0083: this stage is met when the six visibly-off cells above (m9-f-dist-rt A16, A20;
m9-legacy-fdist A47, A48, A55, A56; t1049-fdist A16) are within half a unit of the 15th
significant digit of Excel's value, and no other cell this task owns is newly visibly off. Bit-level
criteria still unmet at that point move to a follow-up task without the campaign key.

## Deferred (decision 0114)
`defer: phase-1b`: the cells this task owns are already within 15 significant digits of Excel, so it waits until Phase 1 exits. Remove the key only when the steward records the Phase 1 exit, or when an eval shows one of its cells visibly off (then give it a `campaign:` stage instead).

## Phase 1 triage (T-0090, 2026-10-27)
110 visibly-off F.DIST cells of probes-numeric/t2220-f-dist-sweep-cdf (and the other t2220 sweeps) are not last-digit residue; T-2981 owns them and routes them to the kernel task that fixes them (T-2219 / T-2790 first guess). This task stays `defer: phase-1b` for its bit-level fits.

## Cells added by gap triage (2026-10-30)
`make eval` on origin/main 6a26b992a (excel-graph), `scripts/exit_owners.py`'s `owned` rule over every failing cell and the task files of origin/main and every pushed branch: 35 F.DIST, F.DIST.RT and FDIST cells within 15 digits (31 within 4 ulp, 4 further; none visibly off) that no open task names, on the m9, m9-legacy and t1049 probes this task already reads and six gen-m9 cells. They include the d1 = d2 = 1 points this task's Context cites (`F.DIST.RT(1,1,1)` and `FDIST(1,1,1)`: Excel 0.4999999999999999, ours 0.5000000000000002; statistics.md "Incomplete beta in the ironcalc-calc workbooks: F tails and densities"). Other examples, Excel vs ours: m9-legacy-fdist A46 `F.DIST.RT(3.5,10,10)` 0.030373282606640854 vs 0.030373282606640812; t1049-fdist A33 `F.DIST(1,1.5,7,TRUE)` 0.6493833371797926 vs 0.6493833371797924; gen-m9/0a614cfe974b59c3 A19 `F.DIST.RT(10,3,4)` 0.024910273396826055 vs 0.024910273396826058.
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A175
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A19
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A325
- gen-m9/4ab2ba98d26c8a5e.case.json (1): Calc!A157
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A121
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A213
- probes-numeric/m9-f-dist-rt.case.json (3): Calc!A2, A3, A18
- probes-numeric/m9-legacy-fdist.case.json (20): Calc!A31, A37, A39, A46, A49, A50, A51, A57, A58, A59, A60, A61, A66, A67, A68, A71, A76, A77, A78, A79
- probes-numeric/t1049-fdist.case.json (3): Calc!A13, A19, A33
- probes-numeric/t1049-fdistrt.case.json (3): Calc!A10, A11, A26
