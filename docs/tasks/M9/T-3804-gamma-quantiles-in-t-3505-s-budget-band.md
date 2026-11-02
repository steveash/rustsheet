---
id: T-3804
title: "Gamma quantiles in T-3505's budget band (shapes 2.9E5 to 1E8): the last ulps of 292 cells of its probe after T-3409's search model"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3409]
defer: phase-1b
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV at shapes 2.9E5 to 1E8 return Excel's double on the 292
known failures of T-3505's budget-band probe listed below (the ones T-3294 does not list), because the engine follows Excel's
inverse search there (T-3409's model) over Temme's one-term cdfs. Otherwise the Outcome gives the bit-exact counts of
each candidate (decision 0056 time box).

## Context
- Gap triage 2026-11-02 (`make eval` on origin/main 56917e82c). The case has 297 known failures in
  eval/baseline/probes.txt. T-3294 lists 5 of them by cell: the visibly-off `GAMMA.INV(1E-300,shape,1)` cells A1, A13,
  A85, A97 and A145, the deep lower tail. The other 292 belong to no task. T-3294 names the case only in a sentence
  about those 5, and `exit_owners.py`'s `owned` rule reads that line as owning the whole case, which hides them.
  T-3505 (done) made the probe. Its Outcome counts 228 bit-exact cells, 291 within 4 ulp and 6 outside that.
- The 292 cells are all within 15 digits, so this is Phase 1b work (`defer: phase-1b`). By function and distance:
  GAMMA.INV 131 at 1 ulp, 3 at 2-4 ulp and A133 at 5 ulp (`GAMMA.INV(1E-300,20000000,1)`); CHISQ.INV.RT 53, CHIINV 53,
  CHISQ.INV 46 and GAMMAINV 5, all at 1 ulp. Examples (Excel, ours):
  - `CHISQ.INV.RT(1E-10,600000)` (S!A260): 606994.8324729953, 606994.8324729952;
  - `GAMMA.INV(0.5,20000000,2.5)` (S!A241): 49999999.16666667, 49999999.16666666;
  - `GAMMA.INV(0.1,7500000,1)` (S!A78): 7496490.540689174, 7496490.540689172.
- Mechanism: from shape 1E5 every cdf of an inversion is Temme's expansion, one term each (`QUANTILE_TEMME_SHAPE_MIN`,
  `incomplete_gamma.rs` `temme_pair`; statistics.md, the T-1891/T-3350 paragraph of the incomplete-gamma budget entry).
  The 1-ulp pattern on both tails and at p = 0.5 points to where the search stops, not to the cdf. T-3409 is modelling
  Excel's start, step rule and stopping point at a huge shape, so this task waits for it. Score that model here
  first, before fitting anything new.
- Cells (every known failure of the probe except T-3294's five):
  - probes/t3505-gamma-inv-budget-band.case.json (292): S!A5:A9, A11:A12, A14, A16, A19, A25:A26, A28, A30, A32, A36:A41, A44, A46:A48, A50:A51, A55:A58, A61:A62, A64:A66, A70, A72, A77:A78, A81:A84, A86:A88, A91, A96, A98:A99, A101, A103:A104, A107, A110:A114, A117, A119, A121, A123, A127:A134, A137, A140:A141, A143:A144, A147:A151, A153:A156, A158:A160, A164, A169, A171, A173:A178, A180, A182:A183, A190, A192, A194, A196, A199:A200, A202:A203, A205, A207:A210, A212, A215:A216, A218, A221:A222, A224:A225, A232, A236, A238:A245, A248:A258, A260:A262, A275:A276, A280, A283:A285, A287:A289, A292, A295:A297, A299:A300, A302:A304, A310, A313:A315, A320:A325, A334:A336, A338:A339, A344:A355, A361:A373, A380:A384, A388:A391, A395:A396, A398:A399, A403:A405, A409, A412:A414, A416:A418, A421:A424, A430:A432, A437:A438, A440:A441, A443:A447, A455:A460, A467:A468, A472:A474, A476:A480, A482:A487, A490:A492, A494:A495, A497:A498, A500:A508, A511:A513, A515:A520, A524:A525
- Neighbours, which do not list these cells: T-3272 (shapes from 1E9, the closed form and band crossings), T-3409
  (t3350, t3070 and t2982 probes), T-3286 (incomplete gamma near the mode, accuracy).

## Acceptance criteria
- [ ] Each of the 292 cells is Excel's double, or the Outcome gives the bit-exact and within-4-ulp counts of each
      candidate tried and lists the cells left, with Excel, ours and the 50-digit math (the Gate A table of T-3529 or
      mpmath).
- [ ] Any fitted change meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
      - hold-out counts stated before scoring (split by cell hash, about 30%, fixed before scoring);
      - a fresh sweep after the freeze (a new probes-numeric case at shapes 2.9E5 to 1E8, log-uniform in shape and p);
      - the Gate A envelope result;
      - the `FITTED` tag with its range;
      - the reproducibility floor (1-ulp web-vs-desktop differences are not targets on their own);
      - no keep-matched branch.
- [ ] The known-failure lines of the cells above in eval/baseline/probes.txt only go down, with no
      new ones in any suite (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The five deep-lower-tail cells (T-3294) and shapes from 1E9 (T-3272).
