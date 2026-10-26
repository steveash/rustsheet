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

## Acceptance criteria
- [ ] These 31 known-failure lines disappear from eval/baseline/gen-m9.txt (`make eval-baseline`) with no new ones in any
      baseline, or the Outcome records the candidates scored with the f-test fit and their bit-exact counts (decision
      0056 time box) and the cells still open move to a follow-up.
- [ ] statistics.md "F.TEST / FTEST: larger variance on top" cites these cells for the model that matches them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 8 visibly-off cells (T-2681) and F.DIST's own last digits (T-2221).
- F.TEST error choice over overflowing squares (T-2432) and one-value arguments (T-2576).
