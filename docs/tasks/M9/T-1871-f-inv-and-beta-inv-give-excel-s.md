---
id: T-1871
title: "F.INV and BETA.INV give Excel's values where statrs' inversion saturates, panics or never finishes"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
F.INV (and BETA.INV where its shapes are extreme) gives Excel's value at a tiny probability where the statrs inversion
the engine uses saturates, panicked or never finished, instead of `#NUM!` or a wrong number.

## Context
Found by T-1848, which capped statrs' `inv_beta_reg` (`statistical/inverse_beta.rs`) so that it is `#NUM!` where statrs
panicked or hung (statistics.md "Quantile searches: per-call cap and lifting", a DEVIATION). Excel's values,
probes/stat-quantile-lift (excel-graph, 2026-10-18):
- E1-E5, E12: `F.INV(p,5,5)` follows the power law 5.081599921802649E-09 * (p/1E-20)^0.4: `F.INV(1E-300,5,5)` =
  5.081599884913164E-121 (ours `#NUM!`, statrs panicked), `F.INV(1E-170,5,5)` = 5.081599884913239E-69 (ours `#NUM!`),
  `F.INV(1E-160,5,5)` = 5.081599884913189E-65 and `F.INV(1E-100,5,5)` = 5.081599884913108E-41 (ours, as on main,
  1.3470392369521052E-15: statrs saturates).
- E6: `F.INV(1E-50,6000,4)` = 0.0004924127886173762 (ours `#NUM!`, statrs never finished).
- E7: `F.INV(1E-50,2000,2)` = 0.004452511009947361; ours, as on main, 0.008195481623831033 (84% off: statrs converges
  to a wrong root at a tiny p).
- Last-digit misses near the same searches, ours as on main: E8 `F.INV(3.718810279553989E-162,4532,43)` =
  0.043055297894031924 (ours 0.04305529789403196, 5 ulp); E10 `F.INV.RT(1-2^-53,6000,4)` = 0.04910526649490917 (ours
  0.049105266494906694, 357 ulp); E11 `FINV(1-2^-53,5,5)` = 2.1094279171713594E-07 (ours 2.1094279171713626E-07,
  12 ulp); E12 `F.INV(1E-20,5,5)` = 5.081599921802649E-09 (ours 5.081599921802652E-09, 4 ulp), the power law's anchor.
- E9: `BETA.INV(1-2^-53,1.0024017013258346,12177.162026553196)` = 0.0691959487211844 (ours, since T-2119, 0.0030131411834558195; it was `#NUM!` when statrs never
  finished); the true quantile is about 0.003, so this is Excel's erratic extreme-shape behaviour (statistics.md
  "Inverse CDF functions at a tiny or near-1 probability (T-1543)").
A lower-tail form of the F quantile (the incomplete beta's leading term, as `t_dist::t_tail_quantile` does for Student's
t) or a bisection on [0, 1] (`special::refine_increasing` with no start, which BETA.INV uses since T-2119) are candidates; T-1745 found Excel's BETA.INV
solves its own cdf to the crossing. Keep the per-call caps and the lift weights of kinds.rs `QUANTILE_WEIGHT`.

gen-m9 sweep (gap triage 2026-10-23, T-2255). F.INV, FINV, F.INV.RT and BETAINV at a tiny or near-1 probability
(p < 1E-8 or p > 1 - 1E-8): ours `#NUM!` (58 cells, e.g. `F.INV(1E-200,1,1)` Excel 2.705579217114786E-151) or a value off in
the 6th to 15th digit (60 cells, e.g. `F.INV(1E-10,1,9)` Excel 1.660345283844152E-20, ours 1.0729605309703607E-15).
Owned gen-m9 cells (T-2255 triage, excel-graph, 2026-10-22): 118 known-failure lines in eval/baseline/gen-m9.txt, 118 formulas.
Examples:
- gen-m9/05d44f6dd6cfc567 Calc!A145 `=F.INV(1E-200,Data!D$11,9)`: Excel 9.999999999999846e-201, ours #NUM!
- gen-m9/06b4e8f5de409d5e Calc!A96 `=F.INV(1E-16,10,6)`: Excel 0.00020601848249191356, ours 0.00020601848249191513
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A145
- gen-m9/06b4e8f5de409d5e.case.json (2): Calc!A96, A149
- gen-m9/0a614cfe974b59c3.case.json (4): Calc!A4, A24, A268, A329
- gen-m9/0a9226016e8f5f33.case.json (4): Calc!A45, A235, A314, A321
- gen-m9/146b46e74f2dac3b.case.json (3): Calc!A117, A367, A375
- gen-m9/165db58d1540f014.case.json (3): Calc!A104, A272, A364
- gen-m9/27601819397dd56e.case.json (2): Calc!A56, A284
- gen-m9/2801c1c11c21e134.case.json (2): Calc!A199, A376
- gen-m9/285d87cea1aaa73c.case.json (2): Calc!A212, A287
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A64, A394
- gen-m9/3a25a03eb94fcfbd.case.json (1): Calc!A44
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A283
- gen-m9/4ab2ba98d26c8a5e.case.json (4): Calc!A110, A240, A248, A332
- gen-m9/4e55500bb8ad3cd8.case.json (4): Calc!A77, A169, A353, A360
- gen-m9/530c636749a85209.case.json (1): Calc!A83
- gen-m9/589f42e67f4b4b8d.case.json (2): Calc!A47, A277
- gen-m9/58e3253bbd3a7a82.case.json (3): Calc!A155, A247, A385
- gen-m9/60b0b6e69ad35a45.case.json (1): Calc!A319
- gen-m9/623663cacea53bbc.case.json (1): Calc!A148
- gen-m9/7674c89c824aca43.case.json (1): Calc!A329
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A28
- gen-m9/7c20351562a2d71d.case.json (6): Calc!A70, A83, A129, A208, A264, A267
- gen-m9/85f48d5cdfb24555.case.json (3): Calc!A10, A12, A256
- gen-m9/899f9555e67da2d6.case.json (3): Calc!A46, A187, A233
- gen-m9/8d3588d8df0812df.case.json (2): Calc!A127, A245
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A296
- gen-m9/a013a45a8fbcab4f.case.json (2): Calc!A20, A81
- gen-m9/a54c738c697617c3.case.json (3): Calc!A94, A338, A386
- gen-m9/a612e89321a6df9c.case.json (6): Calc!A206, A252, A256, A259, A298, A348
- gen-m9/a61e547a1cab994c.case.json (1): Calc!A358
- gen-m9/a790a4d4a6a82756.case.json (3): Calc!A40, A116, A238
- gen-m9/a7bf0d8d81d33ba6.case.json (2): Calc!A181, A333
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A99
- gen-m9/b4173fe25109a238.case.json (3): Calc!A67, A140, A311
- gen-m9/b8ee55934c5b824c.case.json (2): Calc!A34, A334
- gen-m9/bbdfc06cae0e0d71.case.json (3): Calc!A247, A262, A308
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A311
- gen-m9/c18dbf21f11093f3.case.json (2): Calc!A230, A374
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A389
- gen-m9/c468f5046aadfbbc.case.json (2): Calc!A58, A362
- gen-m9/c6cfda6bd10cae93.case.json (5): Calc!A53, A61, A114, A175, A297
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A237
- gen-m9/cde551ea787f713a.case.json (2): Calc!A139, A367
- gen-m9/d8266bc15533702a.case.json (1): Calc!A57
- gen-m9/e6cf3b20a014e6a3.case.json (6): Calc!A13, A74, A143, A257, A290, A379
- gen-m9/eced4bb6a0234c9e.case.json (2): Calc!A203, A335
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A100
- gen-m9/f2bd5f380ec002bf.case.json (3): Calc!A44, A213, A320
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A295
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A209
- gen-m9/fec39efaba9284c3.case.json (3): Calc!A14, A76, A206
- Cells this task owns that an earlier owner rule credited to it or to another task only by an A1
  name written beside a different case (decision 0159, T-0099):
  - gen-m9/7c20351562a2d71d.case.json: Calc!A37, A254
  - gen-m9/c18dbf21f11093f3.case.json: Calc!A190
  - gen-m9/589f42e67f4b4b8d.case.json: Calc!A319
  - gen-m9/8dd2312c96effb1b.case.json: Calc!A169
  - gen-m9/c2974c061384fdd9.case.json: Calc!A267

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2255): these 118 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] probes/stat-quantile-lift E1-E7 pass or are within 4 ulp (named in the Outcome), and E8, E10-E12 pass or are
  no further off than today; E9 matches or its rule is
  recorded as UNVERIFIED in docs/excel-semantics/statistics.md.
- [ ] The values probes-numeric and ironcalc-calc check for F.INV and BETA.INV are unchanged or better; the DEVIATION
  entry shrinks to what remains.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2255): the gen-m9 cells listed under Context are visibly off (a wrong type, error or
text, or a number further than half a unit in the 15th significant digit from Excel's), so the task is in the campaign. Its
target is that every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th
significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states
it. Bit-level criteria still unmet then move to a follow-up task without the key.
