---
id: T-3269
title: "POISSON cdf: fit Excel's deep upper tail (mean well above k)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: M
tier: opus
depends_on: [T-3289]
campaign: visible
---

## Goal
POISSON and POISSON.DIST's cdf match Excel in the deep upper tail, where the mean is well above k (Q(k + 1, mean) with mean >= about 4(k + 1)), without losing any Excel-computed cell of CHISQ.DIST.RT, CHIDIST or GAMMA.DIST that shares the kernel.

## Context
T-2055 fitted the pmf (docs/excel-semantics/statistics.md "POISSON.DIST pmf: two closed forms") and left the cdf on
`statistical/incomplete_gamma` `gamma_ur`. In the deep upper tail Excel is up to 34 ulp from the exact value and up to
60 from ours, with no shared pattern: `POISSON(2,100,TRUE)` = 1.8976107553682424E-40 (gen-m9 c7664e494e418395 A294; ours -35
ulp), `POISSON(3,100,TRUE)` (+60), `POISSON(6,100,TRUE)` (583ca2baea0bb30a A399, -34), `POISSON(9,100,TRUE)` (+44),
`POISSON(6,180,TRUE)` (+24), `POISSON(3,42,TRUE)` (+10). Not the pmf times its finite series 1 + k/m + ...: with Excel's own
pmf that is 45 ulp off at (2, 100). Not exp(fl(a LN x - x - GAMMALN(a))) times the series in any of 48 storage orders
tried (T-2055 scratch fit). This is the shared incomplete-gamma kernel's Q prefactor; coordinate with T-2638 and T-3060.

Stage (decision 0114): `campaign: visible`. The cdf evidence is mixed: away from the deep upper tail Excel is within
about 2 ulp of the exact value (the `accuracy` premise holds there, and the accurate kernel already matches), but in the
tail these cells are, Excel is the inaccurate side (4 to 34 ulp from the exact value at mean 100, k 1 to 5), so matching them
means reproducing Excel's own prefactor, not a more accurate kernel.

Gap triage 2026-10-30 (`make eval` on origin/main 73b187224; `scripts/exit_owners.py --all-off` listed these with no
open owner). The same deep-tail Q(a, x) shows in CHISQ.DIST.RT and CHIDIST, so this task now owns those cells too;
ulps of Excel's value against 60-digit mpmath, Excel / ours:
- POISSON: `POISSON(1,100,TRUE)` -12.1 / +0.9; `POISSON(3,42,TRUE)` -10.9 / -0.9; `POISSON(9,100,TRUE)` -43.3 / +0.7;
  `POISSON.DIST(3,100,TRUE)` (x = 3.14159 truncated) -59.4 / +0.6; `POISSON(8,42,TRUE)` +22.6 / +0.6;
  `POISSON(2,100,TRUE)` (873f30d2bf76f8db, k from ERROR.TYPE) +34.2 / -0.8; `POISSON(6,180,TRUE)` (mean from COUPDAYS)
  -25.3 / -1.3; and gen-m9/3c06f9298f65a3ba A91, a CUMIPMT whose rate is 1/`POISSON.DIST(10,42,TRUE)` (3.4E-15
  relative off; if the cdf fix does not move it, probe the sub-expression before claiming it).
- CHISQ.DIST.RT / CHIDIST: `(200,5)` -32.3 / -0.3; `(1000,5)` -43.1 / -3.1; `(200,20)` = `POISSON(9,100,TRUE)`
  -43.3 / +0.7; `(1000,3)` +86.7 / -1.3. Also off but owned elsewhere: `CHIDIST(1000,30)` +405 (m9-legacy-chidist A71,
  A72), and `CHIDIST(100,4)` -38.1 and `(100,5)` +33.8 (gen-m9 3a25a03eb94fcfbd A158, bbdfc06cae0e0d71 A125), which the
  gap-triage-2026-10-30d branch hands to T-3219; whichever task lands Excel's deep-tail prefactor first takes the others'
  deep-tail cells with it (record the split in both).
- Needs an oracle sweep: these points have "no shared pattern" (above) and 48 storage orders already failed, so the
  fit waits for T-3289's dense sweep (POISSON k 0 to 19 at mean 4(k + 1) to 100(k + 1), the CHISQ.DIST.RT twins,
  half-integer shapes and ulp runs).

Cells (the gen-m9 and probe cells above):
- gen-m9/097af1ca33638670.case.json (1): Calc!A214
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A214
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A80
- gen-m9/9735c5ab72fa5ecf.case.json (1): Calc!A48
- gen-m9/09b47629f54b9817.case.json (1): Calc!A349
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A124
- gen-m9/d208bbafc3bc1997.case.json (1): Calc!A169
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A91
- probes-numeric/m9-legacy-chidist.case.json (4): Calc!A61, A62, A69, A70
- probes-numeric/t1049-chisqdistrt.case.json (3): Calc!A27, A28, A39

Gap triage 2026-10-31c (`make eval` on origin/main 0c917f9fe, excel-graph): eight more deep-tail cdf cells that no open
task listed (each mean is at or near 4(k + 1) or above; the cumulative flag is truthy in each). One is visibly off,
gen-m9/43c6227e0ff368c7 A108 `=POISSON(5,Data!A8,(-1))` with Data!A8 = 42: Excel 7.083529869768314E-13, ours
7.083529869768309E-13 (-5 ulp). The others are 3 to 7 ulp off: `POISSON.DIST(4,100,TRUE)` 1.6139305336977313E-37 vs
1.6139305336977303E-37 (-5); `POISSON.DIST(5,100,TRUE)` (-7); `POISSON(4,30,TRUE)` 3.6243009520614882E-09 vs
3.624300952061487E-09 (-3); `POISSON(8,35,TRUE)` (2138b909d8f810ea A62, mean Data!C12/Data!E2 = 7/0.2) (-7);
`POISSON.DIST(5,105.14159,TRUE)` (201d67f1d59436ed A173) (+4).
- gen-m9/201d67f1d59436ed.case.json (1): Calc!A173
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A62
- gen-m9/43c6227e0ff368c7.case.json (1): Calc!A108
- gen-m9/46ac3ef5e61583ca.case.json (1): Calc!A240
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A394
- gen-m9/5919660b7bb4d195.case.json (1): Calc!A316
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A289
- gen-m9/afc00c8b70e78baf.case.json (1): Calc!A304

## Acceptance criteria
- [x] (Amended by owner ruling (b) and its extension, decision 0203: the nine cells it names are accepted and go to
      T-3469.) The POISSON, CHISQ.DIST.RT and CHIDIST cells above are within 15 digits of Excel (gen-m9/43c6227e0ff368c7 A108 included), and no Excel-computed cell of POISSON, POISSON.DIST,
      CHISQ.DIST.RT, CHIDIST, GAMMA.DIST or GAMMADIST goes from pass to fail (main-vs-HEAD per cell).
- [x] `make check` green; reviewers approve.

## Out of scope
- The pmf (T-3247).

T-3289 landed the sweep: probes-numeric/t3284-gamma-deep-upper-tail.case.json (343 cells, filled by excel-graph).

## Findings (attempts 1 and 2)
- Excel's far upper tail is Stirling's ratio prefactor sqrt(a / 2 pi) exp(a ln(x/a) + (a - x) - correction(a)) (T-2638's
  large-shape form) times the continued fraction, at shapes below 20 for x > 30; at and below x = 30 it is the accurate Q
  at integer shapes. statistics.md "Incomplete gamma: accurate kernel (T-0971)" has the evidence. New probes (excel-graph,
  no pending cell): probes-numeric/t3269-gamma-tail-switch (145 cells, x 26 to 40 at shapes 2 to 4; the switch lies
  between 30 and 30.5), t3269-shape6-misses (77), t3269-fresh-sweep (240, docs/research/t3269/fresh_sweep.py).
- Attempt 1 (the side branch): `incomplete_gamma/excel_tail.rs`, that prefactor with the exact correction at each
  multiple of 1/2 (docs/research/t3269/stirling_correction.py) times T-3285's double-double fraction, for 30 < x <= 600
  (`FITTED(T-3269, 145 cells, x 26..40)` on the switch), used by POISSON's cdf and CHISQ.DIST.RT.
  - R4 hold-out (502 literal cells with a < 20 and x >= a + 1; 30% by hash of (a, x), split fixed before scoring):
    bit-exact train 79 / 345 (22.9%), test 52 / 157 (33.1%); correctly rounded 50 / 345 and 22 / 157; old kernel
    32 / 345 and 19 / 157. Within 4 ulp on test: 147 against 46 (correctly rounded).
  - R4 fresh sweep after the freeze (240 cells, x log-uniform 1 to 1000): far range 20 / 88 bit-exact (old kernel 1,
    correctly rounded 2), 72 / 88 within 4 ulp; elsewhere 74 / 152 (old 73). Gate A envelope: every value within 1e-9 of
    the math (the form is at most a few hundred ulp from it).
  - Eval, main (report of 2026-10-31, 1248554a) vs that branch, per cell (POISSON, CHI*, GAMMA*), with T-3285 included:
    gated fixed 6 (gen-m9 09b47629f54b9817 A349, 583ca2baea0bb30a A399, 8fec7b207f4dd753 A349, bbdfc06cae0e0d71 A125
    `CHIDIST(100,5)`, d208bbafc3bc1997 A169, ironcalc-calc CHISQ.xlsx G3); visibly fixed 13 (also gen-m9 097af1ca33638670
    A214 `POISSON(1,100,TRUE)`, 3a25a03eb94fcfbd A158 `CHIDIST(100,4)`, 3c06f9298f65a3ba A91, 873f30d2bf76f8db A124,
    9735c5ab72fa5ecf A48 `POISSON.DIST(D8,100,..)` 60 to 2 ulp, c7664e494e418395 A294 `POISSON(2,100,TRUE)`,
    f34f66fb7f81e673 A3 `CHIDIST(Data!A2,8)`, CHISQ.xlsx G4); closer 15; farther 4 (5919660b7bb4d195 A80
    `POISSON(9,100,..)` 44 to 72, 43c6227e0ff368c7 A108 `POISSON(5,42,..)` 5 to 25, and T-3285's two 2-to-3-ulp cells);
    lost bit-exact 0; newly visibly off 0. Ungated: fixed 36, visibly fixed 140, closer 130, farther 5, lost bit-exact 2
    (t2561-chisq-dist-rt-past-edge R!A33, A43: 2 and 1 ulp), newly visibly off 2 (R!A73 `CHISQ.DIST.RT(459.29972,12)`
    15 to 129 ulp, R!A75 `(468.485715,12)` 13 to 189). gen-m9 known failures 3015 to 3010; visible parity 2425 to 2413.
  - Not reached: ironcalc-calc POISSON.xlsx D55 (`POISSON.DIST(46040,46071,TRUE)`, near the mode at a huge shape:
    T-3286); gen-m9 68d8ffb69ff739d8 A325 `CHIDIST(COUPNUM(..),5)` unchanged.
- Attempt 2: the shape-6 probe. The form misses only at isolated x at shape 6 (38, 42, 229.64986, 234.2428575) by 1.5
  to 3 ulp of its exponent, while x 0.5 away and shapes 5, 7, 8 at the same x fit; the POISSON twins equal the CHISQ
  cells bit for bit, so it is the kernel. No ln(x/a) or exponent-order variant tried explains it, nor the x / a = 10
  misses (16 to 245 ulp at some shapes).
- Next: an owner ruling (decision 0111) to land the side branch with R!A73 and R!A75 named, or a probe that resolves
  Excel's exponent at shape 6 (dense adjacent doubles around x = 42 and 229.64986).

## Outcome
- Shipped by owner ruling (b) "Ship, record ruling" and its extension "Extend both rulings" (2026-10-31, decision 0203,
  accepted), which name nine cells moved farther (ulps of Excel's value): t2561-chisq-dist-rt-past-edge R!A73 15 to 129
  and R!A75 13 to 189; gen-m9/5919660b7bb4d195 A80 `POISSON(9,100,..)` 44 to 72 and 43c6227e0ff368c7 A108
  `POISSON(5,42,..)` 5 to 25; t3284-gamma-deep-upper-tail A165 / A166 (9 to 90) and A183 / A184 (5 to 79) at x / a = 10;
  gen-m9/caa1eed48b1883fe A59 `CHIDIST(100,9)` 0 to 1 against T-3285's head. T-3469 owns all nine.
- Engine (reworked after review): `incomplete_gamma/excel_tail.rs`, the Stirling ratio prefactor with the exact
  Stirling error (`loader::stirlerr_exact`, gated by `is_half_integer_shape`) times T-3285's double-double fraction,
  at shapes 3/2 to 11 and 30 < x <= 600; `poisson_upper` for POISSON's cdf and CHISQ.DIST.RT's `accurate_tail` take
  it, else the accurate kernel. Fitted pieces (quality/fitted.json, 1 to 2 in the file): `FAR_TAIL_MIN_X` (the switch
  at 30), `FITTED(T-3269, 145 cells, x 26..40, shape 2..4)`, and `FAR_TAIL_MAX_SHAPE` (11),
  `FITTED(T-3269, 153 cells, shape 1.5..19.5, x 30..45)`: next to the cut, on train cells, gaps above 4 ulp, the form
  beats the accurate kernel 80 to 12 at shapes 3/2 to 11 (test 8 to 1) and loses 1 to 3 above (test 2 to 1).
  `FAR_TAIL_MAX_X` (600) is the measured end, UNVERIFIED (T-3469). Rework against the first version: shapes above 11
  go back to T-3285's kernel (bit-identical to T-3285's head there and to the first version at shapes up to 11, on all
  1058 hold-out cells).
- Decision 0188 R4 (docs/research/t3269/: holdout.py, its split holdout-split.tsv, the mpmath values, each binary's
  values and the output holdout.out / fresh2.out):
  - R4.1 hold-out: 1058 literal cells with 1 < a < 20 and x >= a + 1 (the first fresh sweep included, the new one
    not), 30% by `sha256("a,x")` (first 8 hex digits mod 100 < 30), the 145 tail-switch cells forced to train (0 in
    test). Margin: test exact rate within 5 points of train.

    | candidate | train exact | train <=4 ulp | train 15 digits | test exact | test <=4 ulp | test 15 digits |
    |---|---|---|---|---|---|---|
    | head (form at 1.5..11) | 157 / 792 (19.8%) | 591 | 661 | 43 / 266 (16.2%) | 202 | 216 |
    | correctly rounded Q | 89 (11.2%) | 238 | 334 | 24 (9.0%) | 74 | 105 |
    | T-3285 head | 75 | 246 | 369 | 17 | 69 | 110 |
    | main | 73 | 240 | 359 | 17 | 65 | 107 |

    Test against train: 3.6 points, inside the margin; test beats the correctly rounded Q (43 to 24). In the form's
    range alone: test 27 / 157 bit-exact, 150 within 4 ulp (correctly rounded 1 and 19).
  - R4.2 fresh sweep after the freeze: probes-numeric/t3269-fresh-sweep-2 (300 cells, docs/research/t3269/
    fresh_sweep_2.py: a log-uniform 1.5 to 19.5 in halves, x log-uniform 20 to 600, plus 40 cells at x 27 to 34 and
    20 at shapes 10 to 13), filled by excel-graph after the code was final, scored with copied binaries:

    | | exact (all 300) | <=4 ulp | 15 digits | form's range exact (188) | <=4 ulp | 15 digits |
    |---|---|---|---|---|---|---|
    | head | 52 | 207 | 242 | 40 | 163 | 177 |
    | T-3285 head | 15 | 64 | 125 | 3 | 20 | 60 |
    | main | 15 | 64 | 123 | 4 | 20 | 57 |
    | correctly rounded | 3 | 48 | 101 | 1 | 19 | 44 |

    Outside the range (67 cells with x > 30) the unrestricted form would have had 9 bit-exact against the head's 2
    (T-3469 owns finding Excel's switch above shape 11).
  - Gate A: every hold-out and sweep value in the envelope; largest relative error against the 50-digit math 9.8e-14
    (hold-out) and 1.1e-13 (sweep).
- Eval (copied binaries; main 8406c606b, T-3285 head d23e971b8, this head; full runs, per cell):
  - against T-3285's head, gated: fixed 5, closer 14, visibly fixed 11, farther 2 (decision 0203 cells 3 and 4), lost
    1 (cell 9, caa1eed48b1883fe A59), newly visibly off 0. Against main, gated: fixed 8, closer 18, visibly fixed 13,
    farther 6, lost 1, newly visibly off 0 (T-3285's `CHIDIST(42,7)` and four `CHIDIST` cells, decision 0202).
  - `update-baseline` (no `--allow-regressions`; cell 9 was recorded in the first version): every suite unchanged,
    gen-m9 2882.
  - Ungated cells lost or newly visibly off outside decision 0203's first nine cells: the owner's second extension (2026-10-31) covers them, and 0203 lists them (was: listed for an owner ruling, not
    claimed): t2561-chisq-dist-rt-past-edge R!A33 0 to 2 and R!A43 0 to 1; t3269-gamma-tail-switch A80, A100, A136
    (and A99 0 to 1 against main, 1 on T-3285); t3269-shape6-misses A46, A53, A67, A68, A69; t3285-chisq-dist-rt-
    half-integer-sweep A65 6 to 17; t3269-fresh-sweep-2 A70 0 to 3, A205 0 to 32, A165 10 to 25, A173 5 to 17, A209 5 to
    11, A233 6 to 21, A286 2 to 8 (ulps of Excel's value, T-3285's head to this head). The first review's A61, A157,
    A8, A10 of t3269-fresh-sweep and A167, A196 of the half-integer sweep are back on the accurate kernel.
- Follow-up T-3469 (`campaign: visible`): the shape-6 misses, x / a = 10, caa1eed A59, the give-up probe cells, and
  where Excel switches to the form above shape 11.
