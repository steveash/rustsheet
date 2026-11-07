---
id: T-3469
title: "Excel's far-tail Q at shape 6 (x = 38, 42, 229.64986, 234.2428575) and POISSON(9,100) at x/a = 10"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: [T-3269]
campaign: visible
---

## Goal
Excel's far upper tail of Q(a, x) also at the points T-3269's Stirling ratio form misses: shape 6 at isolated x, and
x / a = 10 exactly, so the nine cells decision 0203 names as moved farther (six of them newly visibly off) match Excel again.

## Context
- T-3269 (`incomplete_gamma/excel_tail.rs`) takes Excel's form past x = 30 at shapes below 20: Stirling's ratio
  prefactor sqrt(a / 2 pi) exp(a ln(x/a) + (a - x) - correction(a)) times the accurate continued fraction. It merged by
  owner ruling (b), decision 0203, which names the cells it moved farther:
  - probes-numeric/t2561-chisq-dist-rt-past-edge R!A73 `CHISQ.DIST.RT(459.29972,12)` (15 to 129 ulp) and R!A75
    `CHISQ.DIST.RT(468.485715,12)` (13 to 189), both newly visibly off; shape 6, x = 229.64986 and 234.2428575;
  - gen-m9/5919660b7bb4d195 Calc!A80 `POISSON(9,100,..)` (44 to 72 ulp; x / a = 10) and gen-m9/43c6227e0ff368c7
    Calc!A108 `POISSON(5,Data!A8,(-1))` with Data!A8 = 42 (5 to 25; shape 6, x = 42), both already visibly off.
- probes-numeric/t3269-shape6-misses (77 cells, excel-graph): `POISSON(5,x,TRUE)` for x 220 to 240 and 36 to 48 by 0.5,
  shapes 4 to 7 at the same means. The form misses only at x = 38, 42, 229.64986 and 234.2428575, by 1.5 to 3 ulp of its
  exponent, while x 0.5 away and shapes 5, 7, 8 fit; the POISSON twins equal the CHISQ cells bit for bit, so it is the
  kernel. No ln(x/a) or exponent-order variant tried explains it (statistics.md "Excel's far upper tail (T-3269)").
- x / a = 10 exactly: the form is 16 to 245 ulp from Excel at some shapes (`POISSON(10,110,TRUE)` +90,
  `CHISQ.DIST.RT(390,39)` +245), within 5 at others. T-3269's re-run against main ea7470e4a found four such cells of
  probes-numeric/t3284-gamma-deep-upper-tail newly visibly off, which the extended ruling (b) names (decision 0203):
  Calc!A165 `POISSON(10,110,TRUE)` and A166 `CHISQ.DIST.RT(220,22)` (9 to 90 ulp), Calc!A183 `POISSON(13,140,TRUE)` and
  A184 `CHISQ.DIST.RT(280,28)` (5 to 79).
- Also gen-m9/caa1eed48b1883fe Calc!A59 `CHIDIST(100,9)` = Q(4.5, 50): Excel is the correctly rounded Q (T-3285 has it)
  and the form is 1 ulp off, a gated regression the extended ruling (b) accepted (decision 0203; T-3269 recorded it in
  eval/baseline/gen-m9.txt). Find where Excel leaves the form at
  half-integer shapes past x = 30.
- Lead: a dense probe of adjacent doubles around x = 42 and 229.64986 at shape 6, and around x = 10a, to find where
  Excel's exponent rounds differently (an x87 extended step, or a branch at x = 10a).

- T-3269's rework (decision 0188 R4.3) applies the form only at shapes 3/2 to 11: above 11 the hold-out's cells next
  to x = 30 are not the form (docs/research/t3269/holdout.out), yet far from the cut the form still beats the accurate
  kernel there (hold-out, x > 30 outside the form's range: 55 bit-exact of 164 train cells against 27; fresh sweep
  probes-numeric/t3269-fresh-sweep-2: 9 of 67 against 2, 54 within 4 ulp against 10). Find where Excel switches to it
  above shape 11 (and past x = 600, where `FAR_TAIL_MAX_X` is UNVERIFIED), with a probe like t3269-gamma-tail-switch
  at shapes 12 to 19.5; any widening passes R4 again. Gap triage 2026-11-01 moved this placement (above shape 11 and
  past x = 600) and its 37 t3269-fresh-sweep cells to T-3640 (sweep T-3639); this task keeps shape 6 and x/a = 10.
- Probe cells the form gives up against T-3285's accurate kernel (ungated, ulps of Excel's value, T-3285's head to
  T-3269's), recorded here as T-3269's reviewers asked: t3269-gamma-tail-switch Calc!A80 `POISSON(2,36.5,TRUE)` 0 to 1,
  A100 `CHISQ.DIST.RT(64,7)` 0 to 2, A136 `POISSON(3,35.5,TRUE)` 4 to 29; t3269-shape6-misses Calc!A46
  `POISSON(5,38,TRUE)` 11 to 26, A53 `POISSON(5,41.5,TRUE)` 0 to 4, A67 `POISSON(5,229.64986,TRUE)` 15 to 129, A68
  `POISSON(5,234.2428575,TRUE)` 13 to 189 (the POISSON twins of R!A73 and R!A75), and A69 `POISSON(4,42,TRUE)` 3 to 6,
  which is shape 5, not 6.

Cells this task owns:
- probes-numeric/t2561-chisq-dist-rt-past-edge.case.json (2): R!A73, A75
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A80
- probes-numeric/t1049-chisqdistrt.case.json (1): Calc!A28 `CHISQ.DIST.RT(200,20)`, the twin of A80 (Q(10, 100), x / a = 10;
  gap triage 2026-11-01: visibly off and named by no open task)
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A108
- probes-numeric/t3284-gamma-deep-upper-tail.case.json (4): Calc!A165, A166, A183, A184
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A59 (split with T-2695, deferred, which also lists it: whichever task
  lands first records it in both)

## Acceptance criteria
- [x] The cells above within 15 digits of Excel, or a probe recorded in statistics.md showing why not.
- [x] Any change passes decision 0188 R4 (hold-out split, fresh sweep, FITTED tags), and no cell regresses
      (main-vs-HEAD per cell).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Half-integer shapes on [a + 1, 30] (T-3461). The near-mode window at large shapes (T-3286).

## Second extension (T-3269 rework)
Owns the 19 ungated probes-numeric cells listed in decision 0203 under "Second extension" (lost bit-exact or newly visibly off when the far-tail form was restricted to shapes 3/2 to 11).

## Review notes carried from T-3269 (fidelity round 3)
- docs/research/t3269/holdout.py builds the 50-digit reference from the decimal x string, not the double Excel receives; at large x the "correctly rounded" baseline is off by up to about 130 ulp (e.g. (11.5, 553.155)). Recompute with `mpmath.mpf(float(...))` and regenerate holdout-mpmath.json, fresh2-mpmath.json, the .out files and the T-3269/statistics.md tables. Head still wins (test 43 vs 24, sweep 52 vs 16 at the double x).
- `FAR_TAIL_MAX_SHAPE` = 11 rests on 4 train cells near the cut; above shape 11 at 45 < x <= 600 the form beats the accurate kernel (train 88/16, test 35/2), and the unrestricted form beats head on the hold-out (51 vs 43) and sweep (59 vs 52). Place Excel's switch above shape 11 from that evidence instead of keeping the band.
- `FAR_TAIL_MIN_X`'s tag says shape 2..4 but the cut applies at 1.5..11; cite the fresh-sweep-2 cells at x 27..34, shapes 5..11, or widen the tag.
- R4.2: commit the frozen code before filling a fresh sweep (the T-3269 freeze could not be proven from git).

## Outcome
`excel_far_upper` now takes Loader's direct deviance for the prefactor's exponent instead of the ratio form; the
switch and the shape range are unchanged, so no FITTED cut moves (docs/excel-semantics/statistics.md, "Correction
(T-3469)"). Per cell, main a300350e against the change: 13 cells fixed, 29 closer, 0 farther. Every cell under "Cells
this task owns" is now within 15 digits of Excel: A80, A108 and t1049 A28 bit-exact; R!A73 and A75 2 ulp; A165 and
A166 1 ulp; A183, A184 and caa1eed48b1883fe A59 unchanged (5, 5 and 1 ulp). POISSON passes Gate B (464 to 465/506).
Of the second extension's 19 cells, still visibly off: t3269-shape6-misses A69 `POISSON(4,42,TRUE)` (6 ulp) and
t3269-fresh-sweep-2 A173 and A233 (shape 3/2, 17 and 21 ulp), which move to T-4954.

Adopted as an orphan PR (#3905) by the opus worker, which closed the task on this change: every owned cell is
within 15 digits, and the three remaining Second-extension cells moved to T-4954. The change is a choice of formula
already used by the cdf and density (`direct_deviance`), not a scored fit: no constant moves, no FITTED tag.

Decision 0188 R4 for this structural-model choice (kind (c); review round 1). Columns are n, bit-exact, within 4 ulp,
within 15 digits, inside the Gate A envelope (decision 0188 R2); "ratio" is the old exponent, "direct" this change.
- R4.1 hold-out: T-3269's split unchanged (docs/research/t3269/holdout-split.tsv, 1058 cells, fit set all train);
  ratio = holdout-head.json (T-3269's head), direct = holdout-t3469.json (this branch's binary);
  docs/research/t3469/holdout.out. All cells: train ratio 792 157 591 661 792, direct 792 165 611 678 792; test
  ratio 266 43 202 216 266, direct 266 44 207 221 266. In the form's range (30 < x <= 600, shapes 3/2 to 11):
  train 455 89 411 430 against 455 96 430 446; test 157 27 150 150 against 157 28 155 155. Per cell (ulps of Excel)
  27 closer, 0 farther, 1031 the same (`holdout.py compare head t3469`).
- R4.2 fresh sweep: probes-numeric/t3469-fresh-sweep-3 (200 cells, docs/research/t3469/fresh_sweep_3.py, seed
  34693), committed unfilled in c466178b2 after the freeze a9735e107 (crates/ unchanged since), then filled once from
  excel-graph (2026-11-07). Ratio = the merge base a300350e built in its own worktree, direct = this branch. All
  cells: ratio 200 38 168 174 200, direct 200 39 171 177 200; in the form's range (174 cells) ratio 174 38 159 164
  174, direct 174 39 162 167 174. Per cell 4 closer (A5 `POISSON(9,30.146,TRUE)` 22 to 9 ulp, A51
  `POISSON(9,204.407,TRUE)` 219 to 2, A160 `CHISQ.DIST.RT(287.258,9)` 82 to 0, A200 `CHISQ.DIST.RT(934.622,21)` 506
  to 2), 0 farther, 196 the same. The direct form does not do worse than the ratio form at any hold-out or sweep cell.
- Gate A: all 1058 hold-out cells and all 200 sweep cells are inside the envelope for both forms; no point is
  outside it (largest relative error against the 50-digit Q 9.7e-14 on the hold-out, 9.6e-14 on the sweep).
- The sweep's visibly-off cells now have owners: six inside the form's range, unchanged by this change except A5
  (shape 3/2 A29, A144, A174 at 18 to 28 ulp; shape 5/2 A27, A175 at 5; A5 at 9), to T-4954; sixteen past x = 600
  or above shape 11, where the accurate kernel stays (12 to 901 ulp), to T-3640 (deferred): Excel is not the
  accurate kernel there either, and the shape-3/2 misses are not confined to fresh-sweep-2.
