---
id: T-2765
title: "F.TEST and FTEST last digits on 31 gen-m9 cells (1 to 7 ulp): Excel's two-tailed F at the variance ratio"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2681, T-2221]
defer: phase-1b
---

## Goal
The 31 F.TEST and FTEST cells of the gen-m9 sweep listed below, all already within 15 significant digits of Excel,
match Excel to the bit.

## Context
Gap triage 2026-10-26 (`make eval` on 1c8b3d34, excel-graph gen-m9 evidence). No open task lists these cells: T-2681
owns the 8 F.TEST/FTEST cells still visibly off after T-2571 and is identifying how Excel evaluates the tail
(statistics.md "F.TEST / FTEST: larger variance on top"); T-2221 fits F.DIST/F.DIST.RT's last digits on the T-2220 sweep.
Every cell here is 1 to 7 ulp off and not visibly off, so this is Phase 1b work (decision 0114) and stays deferred. It
depends on T-2681 (the tail form) and T-2221 (the F kernel's last digits), since either moves these cells: re-run the
list after them before any fit.

Root cause (suspected): the same as T-2681's, one level down. T-2571 showed the residual is in Excel's tail evaluation,
not in the variance ratio f nor the remainder; here the tail is right to 15 digits but not to the bit. Score candidates
with the committed fit (`cd tools && uv run rustsheet-fitlab f-test`, tools/src/rustsheet_tools/fitlab/kernels/ftest.py)
over these cells plus T-2681's. Numeric fitting, so opus.

Examples (`make eval-show CASE=gen-m9/<id>.case.json`), Excel vs ours:
- gen-m9/f2bd5f380ec002bf Calc!A335 `=FTEST(Data!A10:D11,Data!B5:D10)`: 0.25103524528225196 vs 0.2510352452822516 (7 ulp)
- gen-m9/b6593d7dd13f2dac Calc!A161 `=FTEST(Data!A1:D12,Data!A9:D11)`: 4.5830554353611047E-23 vs 4.583055435361109E-23 (7 ulp)
- gen-m9/f34f66fb7f81e673 Calc!A188 `=FTEST(Data!A1:D5,Data!B3:E9)`: 0.13858402643062293 vs 0.13858402643062276 (6 ulp)
- gen-m9/b8ee55934c5b824c Calc!A327 `=F.TEST(Data!C1:D9,Data!C10:E10)`: 2.5992419872168183E-15 vs 2.5992419872168163E-15 (5 ulp)
- gen-m9/cde551ea787f713a Calc!A321 `=FTEST({100000000.1,100000000.2,100000000.3},{1,"a",TRUE,3})`: 0.009925619873629894 vs
  0.009925619873629896 (1 ulp)
Code: crates/rustsheet-engine/src/functions/statistical/ (F.TEST over the F distribution in `incomplete_beta.rs`; fisher.rs
holds the one-degree-of-freedom closed form).

Owned gen-m9 cells (known-failure lines in eval/baseline/gen-m9.txt), each with F.TEST or FTEST as the outer call:
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A219
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A306
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A192
- gen-m9/40425341c5034eed.case.json (1): Calc!A194
- gen-m9/58e3253bbd3a7a82.case.json (1): Calc!A371
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A221
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A87
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A398
- gen-m9/899f9555e67da2d6.case.json (4): Calc!A39, A177, A229, A315
- gen-m9/b6593d7dd13f2dac.case.json (2): Calc!A161, A283
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A327, A347
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A10
- gen-m9/bc40d37cb4d8301d.case.json (2): Calc!A31, A336
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A19
- gen-m9/caa1eed48b1883fe.case.json (2): Calc!A136, A224
- gen-m9/cde551ea787f713a.case.json (1): Calc!A321
- gen-m9/d8266bc15533702a.case.json (1): Calc!A110
- gen-m9/eced4bb6a0234c9e.case.json (3): Calc!A36, A312, A358
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A105, A243, A335
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A188

T-2676 sweep (gen-m9 round 2, gap triage 2026-10-26, excel-graph): 1 more cell, an FTEST nested in ACCRINT, not
visibly off. gen-m9/932b33b1ef8b7608 Calc!A168 `=ACCRINT(33527,35220,36906,FTEST(Data!A1:A12,Data!A5:C6),3572,2,4)`:
Excel 6831.861360134758, ours 6831.861360134756. ACCRINT is linear in the rate here (`ACCRINT(33527,35220,36906,1,3572,2,4)`
is 33031.07777777778 in ours) and our FTEST is 0.206831318254199; the rate two ulps up, 0.20683131825419906, gives Excel's
value in every product order, so the innermost difference is FTEST, about 2 ulp low (the same class as the cells above).
- gen-m9/932b33b1ef8b7608.case.json (1): Calc!A168

## Acceptance criteria
- [ ] These 32 known-failure lines (the 31 F.TEST and FTEST cells plus the T-2676 cell 932b33b1ef8b7608 A168) disappear from eval/baseline/gen-m9.txt (`make eval-baseline`) with no new ones in any
      baseline, or the Outcome records the candidates scored with the f-test fit and their bit-exact counts (decision
      0056 time box) and the cells still open move to a follow-up.
- [ ] statistics.md "F.TEST / FTEST: larger variance on top" cites these cells for the model that matches them.
- [ ] The 60 cells added by gap triage 2026-10-30 are bit-exact (their 57 gen-m9 lines leave eval/baseline/gen-m9.txt; the probes-numeric cells, ungated, shown with `make eval-show`) or the Outcome lists each with its ulp distance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 8 visibly-off cells (T-2681) and F.DIST's own last digits (T-2221).
- F.TEST error choice over overflowing squares (T-2432) and one-value arguments (T-2576).

## Phase 1 triage (T-0090, 2026-10-27)
The visibly-off F.TEST cell gen-m9/b6593d7dd13f2dac Calc!A306 goes to the T-0087 sweep with the other unowned visibly-off F.TEST/FTEST cells (T-2681, the natural owner, finished on 2026-10-27 before this triage landed); this task keeps the last-digit work.

## Cells added by gap triage (2026-10-30)
`make eval` on origin/main 6a26b992a (excel-graph), `scripts/exit_owners.py`'s `owned` rule over every failing cell and the task files of origin/main and every pushed branch: 60 more F.TEST and FTEST cells within 15 digits (57 within 4 ulp, 3 further; none visibly off) that no open task names: 57 gen-m9 cells with F.TEST or FTEST as the outer call, and three on the t1049 and m9-legacy F.TEST probes. Same root cause as the cells above (Excel's two-tailed F at the variance ratio, now through `f_test_tail.rs` since T-2681), so score them together. Examples, Excel vs ours: gen-m9/7bd219ec0c0c5e22 A313 `F.TEST(Data!D8:D11,Data!B4:C12)` 1.0354587903758923e-14 vs 1.0354587903758912e-14; gen-m9/c6cfda6bd10cae93 A44 `FTEST(Data!C4:D11,Data!D9:E10)` 0.17265211336953595 vs 0.17265211336953612; probes-numeric/t1049-ftest A24 `F.TEST(B1:C2,B3:C5)` 0.04001227468543416 vs 0.04001227468543417.
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A13
- gen-m9/27601819397dd56e.case.json (3): Calc!A130, A298, A314
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A63, A168
- gen-m9/285d87cea1aaa73c.case.json (2): Calc!A158, A280
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A123
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A223
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A297
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A374
- gen-m9/4250f880ab640c49.case.json (1): Calc!A386
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A266, A400
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A322
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A252, A393
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A233, A300
- gen-m9/623663cacea53bbc.case.json (1): Calc!A249
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A21
- gen-m9/7bd219ec0c0c5e22.case.json (2): Calc!A313, A369
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A337
- gen-m9/8d3588d8df0812df.case.json (3): Calc!A60, A243, A359
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A55
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A104, A125
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A75, A139
- gen-m9/a5828d3925575713.case.json (1): Calc!A222
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A10
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A40
- gen-m9/b4173fe25109a238.case.json (1): Calc!A256
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A310
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A377
- gen-m9/c2974c061384fdd9.case.json (2): Calc!A141, A385
- gen-m9/c6cfda6bd10cae93.case.json (4): Calc!A42, A44, A105, A286
- gen-m9/caa1eed48b1883fe.case.json (2): Calc!A319, A380
- gen-m9/cde551ea787f713a.case.json (2): Calc!A34, A199
- gen-m9/d8266bc15533702a.case.json (1): Calc!A354
- gen-m9/e6cf3b20a014e6a3.case.json (2): Calc!A206, A239
- gen-m9/ee2497a5e683c5d9.case.json (2): Calc!A155, A201
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A59
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A279
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A36
- probes-numeric/m9-legacy-ftest.case.json (1): Calc!A31
- probes-numeric/t1049-ftest.case.json (2): Calc!A13, A24
