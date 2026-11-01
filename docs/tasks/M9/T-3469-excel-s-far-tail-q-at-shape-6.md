---
id: T-3469
title: "Excel's far-tail Q at shape 6 (x = 38, 42, 229.64986, 234.2428575) and POISSON(9,100) at x/a = 10"
milestone: M9
lane: functions-stat
status: todo
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
  at shapes 12 to 19.5; any widening passes R4 again.
- Probe cells the form gives up against T-3285's accurate kernel (ungated, ulps of Excel's value, T-3285's head to
  T-3269's), recorded here as T-3269's reviewers asked: t3269-gamma-tail-switch Calc!A80 `POISSON(2,36.5,TRUE)` 0 to 1,
  A100 `CHISQ.DIST.RT(64,7)` 0 to 2, A136 `POISSON(3,35.5,TRUE)` 4 to 29; t3269-shape6-misses Calc!A46
  `POISSON(5,38,TRUE)` 11 to 26, A53 `POISSON(5,41.5,TRUE)` 0 to 4, A67 `POISSON(5,229.64986,TRUE)` 15 to 129, A68
  `POISSON(5,234.2428575,TRUE)` 13 to 189 (the POISSON twins of R!A73 and R!A75), and A69 `POISSON(4,42,TRUE)` 3 to 6,
  which is shape 5, not 6.

Cells this task owns:
- probes-numeric/t2561-chisq-dist-rt-past-edge.case.json (2): R!A73, A75
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A80
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A108
- probes-numeric/t3284-gamma-deep-upper-tail.case.json (4): Calc!A165, A166, A183, A184
- gen-m9/caa1eed48b1883fe.case.json (1): Calc!A59 (split with T-2695, deferred, which also lists it: whichever task
  lands first records it in both)

## Acceptance criteria
- [ ] The cells above within 15 digits of Excel, or a probe recorded in statistics.md showing why not.
- [ ] Any change passes decision 0188 R4 (hold-out split, fresh sweep, FITTED tags), and no cell regresses
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
