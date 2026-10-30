---
id: T-3198
title: "F tails and densities T-2219 left off: F.DIST.RT(3,50,100) on L140 and F.DIST densities at (50,100); T-2219's gen-m9 cells below a + b = 10"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
The F cells T-2219 left visibly off, and its gen-m9 cells, agree with Excel to 15 significant digits.

## Context
T-2219 moved half-integer shapes with 10 <= a + b < MAXGAM, outside the closed forms and the ported
bratio branches, to an accurate double-double kernel (`incomplete_beta.rs::half_integer_tail`;
docs/excel-semantics/statistics.md "Incomplete beta in the ironcalc-calc workbooks", bullet
"Accurate tails at half-integer shapes from a + b = 10"). Cells of its table it did not reach:
- probes-numeric/m9-f-dist-rt A19 `F.DIST.RT(3,50,100)`: Excel 1.516786664496894e-6, ours
  1.5167866644968792e-6 (mpmath 1.5167866644968894697e-6): bratio's dispatch is L140 bup + bpser,
  which is ported (`bratio/bup.rs`), so the error is that branch's (T-2506 family).
- probes-numeric/m9-f-dist A26 `F.DIST(0.5,50,100,FALSE)` (Excel 0.08394970207752739, ours
  0.08394970207752746) and A30 `F.DIST(3,50,100,FALSE)` (Excel 1.05292998522241E-05, ours
  1.0529299852224087E-05): the density is Excel's own chain model (`f_density.rs`, T-2586/T-3034),
  which reaches 15 digits only where |c ln x| is small.
- gen-m9 cells T-2219 owned that stay off: all at a + b below 10 (where T-2219's series only trades
  bit-exact cells, so Cephes stays) or on other paths (df 1E10, shape-of-1 forms). They need
  Excel's own formulation of those branches, so check each against T-2221's fit before working:
  - gen-m9/2801c1c11c21e134.case.json (2): Calc!A219, A374
  - gen-m9/285d87cea1aaa73c.case.json (1): Calc!A34
  - gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A259
  - gen-m9/35e8168aae2b00f8.case.json (1): Calc!A123
  - gen-m9/3a25a03eb94fcfbd.case.json (2): Calc!A49, A181
  - gen-m9/4bad303d32cc3f46.case.json (1): Calc!A141
  - gen-m9/68d8ffb69ff739d8.case.json (3): Calc!A3, A29, A273
  - gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A349
  - gen-m9/8dd2312c96effb1b.case.json (2): Calc!A144, A395
  - gen-m9/8f2ac25945af87da.case.json (1): Calc!A92
  - gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A220
  - gen-m9/a5828d3925575713.case.json (1): Calc!A235
  - gen-m9/a612e89321a6df9c.case.json (1): Calc!A70
  - gen-m9/b6593d7dd13f2dac.case.json (1): Calc!A50
  - gen-m9/b8ee55934c5b824c.case.json (1): Calc!A22
  - gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A344
  - gen-m9/c18dbf21f11093f3.case.json (1): Calc!A289
  - gen-m9/d8266bc15533702a.case.json (2): Calc!A120, A337
  - gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A118
  - gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A151
Many of these are already within 15 digits (bit-level, T-2221's); triage first with
`make eval-show` and a 50-digit reference, and close the ones that are.

## Acceptance criteria
- [ ] m9-f-dist-rt A19 and m9-f-dist A26, A30 within half a unit of the 15th digit of Excel.
- [ ] Every gen-m9 cell above within 15 digits of Excel, or named in the Outcome and handed to the
      task that owns its branch (T-2221 for Excel's own last bits).
- [ ] No gated known-failure line added; `make check` green; reviewers approve.

## Out of scope
- The F quantiles (T-2511) and bit-level fits of Excel's formulation (T-2221).
