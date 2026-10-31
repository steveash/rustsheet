---
id: T-2695
title: "CHIDIST, CHISQ.DIST, CHIINV and GAMMA.INV last digits on the gen-m9 sweep (86 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-2638, T-2367]
defer: phase-1b
---

## Goal
The chi-squared and gamma cells of the gen-m9 sweep that are already within 15 significant digits of Excel match it to
the bit: CHIDIST (36 cells), CHISQ.DIST (27), CHIINV (17), GAMMAINV (3) and GAMMA.INV (3), each with the function as the
formula's outer call.

## Context
Gap triage 2026-10-25 (`make eval`, excel-graph gen-m9 evidence). No open task lists these cells: T-2638 owns the visibly-off
GAMMA.DIST/CHISQ.DIST/CHIDIST/CHIINV tails and densities, T-0972 and T-2367 the gamma quantile and incomplete-gamma
residue they name, and T-2122 the gamma kernels. Every cell here is 1 to 4 ulp off and not visibly off, so this is
Phase 1b work (decision 0114) and stays deferred; it depends on T-2638 and T-2367 because their kernel changes will move
these cells, and the cell list should be re-run after them before any fit.
Examples (`make eval-show CASE=gen-m9/<id>.case.json`), Excel vs ours:
- gen-m9/0a614cfe974b59c3 Calc!A253 `=CHIDIST(37.5,3)`: 3.606484453663215E-08 vs 3.606484453663213E-08
- gen-m9/320d51765af9b6b7 Calc!A157 `=CHIDIST(0.5,8)`: 0.9998666303494859 vs 0.999866630349486
- gen-m9/146b46e74f2dac3b Calc!A148 `=CHISQ.DIST(2,4,Data!$D$10)`: 0.18393972058572114 vs 0.18393972058572117
- gen-m9/146b46e74f2dac3b Calc!A87 `=CHISQ.DIST(DAY(53875),9,0)`: 0.015813618949355984 vs 0.01581361894935599
- gen-m9/27601819397dd56e Calc!A323 `=CHIINV(1E-10,7)`: 60.89519536467025 vs 60.89519536467024
- gen-m9/35e8168aae2b00f8 Calc!A39 `=GAMMA.INV(0.001,Data!A10,100)`: 7.85398574631245E-05 vs 7.853985746312451E-05
Code: crates/rustsheet-engine/src/functions/statistical/ (the incomplete gamma behind CHISQ.DIST and GAMMA.DIST, and the
inversion behind CHISQ.INV and GAMMA.INV). docs/excel-semantics/statistics.md has the current models.

Cells added by gap triage 2026-10-30d. `make eval` on origin/main ran, then the owner rule of
`scripts/exit_owners.py` was applied to every failing cell, with the task files of origin/main and of the pushed task
branches. These three gen-m9 known-failure lines are 1 to 4 ulp off, not visibly off, and no open task names them:
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A268 `CHISQ.DIST(1E-300,COUNTA(Data!$D2),"FALSE")` (shape 0.5, density):
  3.9894228040142567E+149 vs 3.9894228040142563E+149 (-1 ulp)
- gen-m9/c2974c061384fdd9.case.json (1): Calc!A13 `CHISQ.DIST(SUM(ROUND(Data!B$1,-6),Data!E5:E11),10,BYCOL(..))` (cdf):
  0.09849448946078171 vs 0.09849448946078165 (-4 ulp)
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A325 `CHIDIST(COUPNUM(39311,44972,2,"1"),5)`: 9.367248705398532E-06 vs
  9.367248705398537E-06 (+3 ulp)

Owned gen-m9 cells (gap triage 2026-10-25, excel-graph): 86 known-failure lines in eval/baseline/gen-m9.txt, none visibly off.
Cells:
- gen-m9/05d44f6dd6cfc567.case.json (1): Calc!A231
- gen-m9/0a614cfe974b59c3.case.json (2): Calc!A253, A300
- gen-m9/0a9226016e8f5f33.case.json (1): Calc!A399
- gen-m9/146b46e74f2dac3b.case.json (3): Calc!A87, A148, A396
- gen-m9/27601819397dd56e.case.json (2): Calc!A231, A323
- gen-m9/2a18b2f5a4dc43ac.case.json (2): Calc!A324, A383
- gen-m9/320d51765af9b6b7.case.json (2): Calc!A157, A249
- gen-m9/35e8168aae2b00f8.case.json (1): Calc!A39
- gen-m9/3a25a03eb94fcfbd.case.json (5): Calc!A97, A133, A246, A255, A341
- gen-m9/3e98b26cf6713e77.case.json (1): Calc!A329
- gen-m9/40425341c5034eed.case.json (2): Calc!A75, A224
- gen-m9/4250f880ab640c49.case.json (1): Calc!A125
- gen-m9/4bad303d32cc3f46.case.json (2): Calc!A279, A354
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A249
- gen-m9/530c636749a85209.case.json (3): Calc!A12, A90, A150
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A138, A382
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A52
- gen-m9/7674c89c824aca43.case.json (1): Calc!A332
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A101
- gen-m9/7c20351562a2d71d.case.json (1): Calc!A316
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A84
- gen-m9/899f9555e67da2d6.case.json (2): Calc!A51, A118
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A374
- gen-m9/8f2ac25945af87da.case.json (2): Calc!A108, A352
- gen-m9/8fec7b207f4dd753.case.json (2): Calc!A105, A349
- gen-m9/984cba67bbe6972b.case.json (4): Calc!A28, A91, A175, A252
- gen-m9/9cca17438a7e6f8a.case.json (2): Calc!A133, A255
- gen-m9/a013a45a8fbcab4f.case.json (3): Calc!A91, A209, A274
- gen-m9/a54c738c697617c3.case.json (2): Calc!A49, A104
- gen-m9/a612e89321a6df9c.case.json (3): Calc!A173, A181, A227
- gen-m9/a61e547a1cab994c.case.json (2): Calc!A117, A362
- gen-m9/abc2939ebd1e2365.case.json (2): Calc!A57, A107
- gen-m9/b6593d7dd13f2dac.case.json (2): Calc!A253, A265
- gen-m9/b8ee55934c5b824c.case.json (4): Calc!A32, A216, A262, A400
- gen-m9/bbdfc06cae0e0d71.case.json (1): Calc!A268
- gen-m9/bc40d37cb4d8301d.case.json (1): Calc!A165
- gen-m9/bcb24385baafb2f0.case.json (1): Calc!A272
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A301
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A59
- gen-m9/cde551ea787f713a.case.json (1): Calc!A240
- gen-m9/d8266bc15533702a.case.json (2): Calc!A17, A291
- gen-m9/e6cf3b20a014e6a3.case.json (4): Calc!A92, A162, A223, A345
- gen-m9/ecc87f80077959e0.case.json (2): Calc!A170, A212
- gen-m9/f34f66fb7f81e673.case.json (1): Calc!A49
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A236

More last-digit cells of the same kernels (gap triage 2026-10-26, `make eval` on 1c8b3d34, excel-graph), none visibly
off, that no task listed: CHISQ.INV 5, CHISQ.INV.RT 4, CHIDIST 2, CHISQ.DIST 2, GAMMAINV 1 and GAMMADIST 1, each the
outer call (1 to 13 ulp; e.g. gen-m9/8dd2312c96effb1b Calc!A368 `=CHISQ.INV.RT(1-1E-10,4)`: Excel 2.8284405751796042E-05,
ours 2.8284405751796036E-05; gen-m9/85f48d5cdfb24555 Calc!A23 `=CHIDIST(Data!$B$9,MIRR(..))`: Excel 1.5541594313896074E-21,
ours 1.554159431389605E-21). The CHISQ.INV and CHISQ.INV.RT cells are the same gamma inversion as the CHIINV and GAMMA.INV
cells above (T-0972 owns the quantile parity).
Cells:
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A86
- gen-m9/530c636749a85209.case.json (1): Calc!A372
- gen-m9/68d8ffb69ff739d8.case.json (1): Calc!A65
- gen-m9/85f48d5cdfb24555.case.json (1): Calc!A23
- gen-m9/8dd2312c96effb1b.case.json (1): Calc!A368
- gen-m9/a5828d3925575713.case.json (1): Calc!A372
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A348
- gen-m9/caa1eed48b1883fe.case.json (2): Calc!A151, A261
- gen-m9/ecc87f80077959e0.case.json (2): Calc!A2, A119
- gen-m9/eced4bb6a0234c9e.case.json (1): Calc!A150
- gen-m9/ee2497a5e683c5d9.case.json (1): Calc!A136
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A304
- gen-m9/fec39efaba9284c3.case.json (1): Calc!A104

## Acceptance criteria
- [ ] The 86 owned cells listed under Context match Excel to the bit, so these 86 known-failure lines leave
      eval/baseline/gen-m9.txt, with no new line in any baseline; cells that a model cannot reach move to a follow-up with
      the evidence.
- [ ] statistics.md records the model change and cites the cells.
- [ ] The 43 cells added by gap triage 2026-10-30 are bit-exact (their gen-m9 and probes lines leave eval/baseline/; the two probes-numeric cells, ungated, shown with `make eval-show`) or the Outcome lists each with its ulp distance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Visibly-off chi-squared and gamma cells (T-2638), and the GAMMA, GAMMALN and LN kernels themselves (T-2122).

## Phase 1 triage (T-0090, 2026-10-27)
Four visibly-off gen-m9 cells this task named are handed on so this parked task holds only last-digit work: F.INV gen-m9/0a614cfe974b59c3 Calc!A268 and gen-m9/85f48d5cdfb24555 Calc!A12 and FINV gen-m9/85f48d5cdfb24555 Calc!A10 to T-2511; CONFIDENCE.T gen-m9/85f48d5cdfb24555 Calc!A372 to T-0967.

## Cells added by gap triage (2026-10-30)
`make eval` on origin/main 6a26b992a (excel-graph), `scripts/exit_owners.py`'s `owned` rule over every failing cell and the task files of origin/main and every pushed branch: 43 more incomplete-gamma cells within 15 digits (39 within 4 ulp, 4 further; none visibly off) that no open task names. 40 are gen-m9 cells where the function is the outer call or the only non-glue call inside another (CHIINV 7, CHISQ.INV 7, CHISQ.INV.RT 6, GAMMAINV 5, CHISQ.DIST 5, GAMMA.DIST 4, GAMMA.INV 3, CHIDIST 3, GAMMADIST 2, POISSON 1, over the whole group). The other three are round trips on the t1049 probes and one t2436 rerun of a gen-m9 cell. The neighbouring deferred tasks on the unmerged `tasks/m9-opus-fitting-deferred` branch (gamma quantiles and gamma cdf/density on the m9, t1048 and t1049 probes) do not list them. Examples, Excel vs ours: gen-m9/abc2939ebd1e2365 A291 `CHIINV(0.05,CSC((Data!D2-(-0.5))))` 3.8414588206941236 vs 3.841458820694128; gen-m9/2a18b2f5a4dc43ac A333 `GAMMA.DIST(10,DAY(26264),0.999,"FALSE")` 1.1450400571562037e-05 vs 1.1450400571562056e-05; probes-numeric/t1049-chisqinv A33 `CHISQ.INV(CHISQ.DIST(4,3,TRUE),3)` 4.000000000000001 vs 3.999999999999999.
- gen-m9/0620505280d2ea6e.case.json (1): Calc!A292
- gen-m9/0a614cfe974b59c3.case.json (1): Calc!A178
- gen-m9/146b46e74f2dac3b.case.json (1): Calc!A30
- gen-m9/165db58d1540f014.case.json (1): Calc!A375
- gen-m9/285d87cea1aaa73c.case.json (1): Calc!A51
- gen-m9/2a18b2f5a4dc43ac.case.json (1): Calc!A333
- gen-m9/31f09e6c48ca245c.case.json (1): Calc!A191
- gen-m9/320d51765af9b6b7.case.json (1): Calc!A23
- gen-m9/35e8168aae2b00f8.case.json (2): Calc!A158, A196
- gen-m9/38e4fc309fe199ea.case.json (1): Calc!A57
- gen-m9/4250f880ab640c49.case.json (1): Calc!A310
- gen-m9/4bad303d32cc3f46.case.json (3): Calc!A38, A227, A289
- gen-m9/4e55500bb8ad3cd8.case.json (1): Calc!A386
- gen-m9/60b0b6e69ad35a45.case.json (2): Calc!A85, A268
- gen-m9/623663cacea53bbc.case.json (1): Calc!A299
- gen-m9/7bd219ec0c0c5e22.case.json (1): Calc!A9
- gen-m9/8dd2312c96effb1b.case.json (2): Calc!A150, A319
- gen-m9/8fec7b207f4dd753.case.json (1): Calc!A173
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A272
- gen-m9/9cca17438a7e6f8a.case.json (1): Calc!A370
- gen-m9/a54c738c697617c3.case.json (2): Calc!A29, A354
- gen-m9/a612e89321a6df9c.case.json (1): Calc!A273
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A145
- gen-m9/a790a4d4a6a82756.case.json (1): Calc!A257
- gen-m9/a7bf0d8d81d33ba6.case.json (1): Calc!A121
- gen-m9/abc2939ebd1e2365.case.json (1): Calc!A291
- gen-m9/b4173fe25109a238.case.json (2): Calc!A220, A366
- gen-m9/b8ee55934c5b824c.case.json (1): Calc!A42
- gen-m9/c18dbf21f11093f3.case.json (1): Calc!A349
- gen-m9/d8266bc15533702a.case.json (1): Calc!A230
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A231
- gen-m9/f2bd5f380ec002bf.case.json (1): Calc!A293
- gen-m9/fd9fa22d212080d6.case.json (1): Calc!A123
- probes-numeric/t1049-chisqinv.case.json (1): Calc!A33
- probes-numeric/t1049-chisqinvrt.case.json (1): Calc!A33
- probes/t2436/8d3588d8df0812df.case.json (1): Calc!A16
