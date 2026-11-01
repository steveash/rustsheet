---
id: T-3174
title: "F.DIST density after M-final: the constant K where |c·LN x| < 4 (one df of 1 or 2 against a large partner, and (158,450) at x = 1; 220 unowned visibly-off cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
defer: phase-1b
---

## Goal
Where M-final's power term is small (|P| = |c·LN x| < 4, so P's rounding cannot explain the miss), `F.DIST(x,d1,d2,FALSE)`
agrees with Excel to half a unit of the 15th significant digit: the constant K of `excel_f_density` is fitted for
shapes with one df of 1 or 2 against a large partner, and for the large-df cells near x = 1 listed below.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell this task owns is a BETA.DIST or
  F.DIST density cell or a dense probes-numeric sweep cell, within Gate A's envelope (most agree with Excel to 12 or
  more digits, and Excel and the engine are both close to the math), so R1(c) counts it as Phase 1b evidence. A
  point that Gate A's tables show outside the envelope gets its own task without the key.
- Gap triage 2026-10-29 (`make eval` on main a2ecf005b, excel-graph; `python3 scripts/exit_owners.py --all-off`): of the
  18,293 unowned visibly-off F.DIST density cells left after T-3034, 220 have |P| < 4 (M-final's regions: side and r
  from fl(m·x) against n, c = d1/2 on side 1 and -d2/2 on side 2). T-3173 owns the rest (the power term).
- With |P| < 4 the exponent is E = K + P + H with P and H good to well under a unit, so the miss is K. r5-f-final's
  RESULTS.md section 4 found the same: 159 within-15 failures at |P| < 4 on the research rows, 154 of them at small df,
  "almost all shapes with one df of 1 or 2 and a large partner: (160,1), (1,156), (321,2), (472,1). That is r4's known
  small-df constant residue (the stirlerr e-table at e(0.5) and e(1) against a large h), and the rms there is 3-4
  units". T-2586 item 6 lists it as "the +3 to +5 unit offset at d1 = 1, d <= 2 cells".
- Shapes of the 216 small-df cells (count): (1,200) 24, (2,80) 21, (100,2) 20, (100,1) 18, (40,1) 18, (2,500) 13,
  (1766,1) 13, (1,1000) 12, (160,1) 11, (500,1) 11, (1000,1) 11, (472,1) 8, (1,156) 8, (321,2) 7, (848,1) 6, (1,372) 5,
  (500,2) 2, (1,160) 2, and one each at (1,100), (2,100), (3,20), (3,100), (100,3), (1,500). So K's small-df delta
  (r4-f-small-df's M-sd delta: Loader stirlerr error table + bracket + 5.33, `bcmodel.K_of`) has to follow the
  partner df h; the cached cells give 16 shapes with 5 or more points each, enough to fit and hold out by shape.
- The 4 large-df cells (min df >= 16, so K is M-bc's C_BIG branch, "M-sd's side bracket - 0.42"):
  - probes-numeric/t2586-verify-r5-f-bigconst-hold Calc!B253 (x = 1-2^-52, d1 158, d2 450): Excel 3.047040066238604,
    ours 3.047040066238656 (1.7E-14 relative, about 150 units of 2^-53 in E)
  - probes-numeric/t2586-verify-r5-f-bigconst-hold Calc!B254 (x = 1-2^-53): Excel 3.04704006623863, ours 3.047040066238682
  - probes-numeric/t2586-verify-r5-f-bigconst-hold Calc!B508 (x = 1+2^-52, d1 450, d2 158): Excel 3.047040066238603,
    ours 3.047040066238654
  - probes-numeric/t2586-r5-f-smallr-a Calc!B3448 `F.DIST(1.0577004133387637,150,50,FALSE)`: Excel 1.5804742363893771,
    ours 1.5804742363893858
  B253 and B254 differ in Excel by 26E-15 for one ulp of x: r = fl(158x/450) moves one ulp and H = -h·L(r) with
  h = 304 carries it, which M-final reproduces; the common 5.2E-14 offset is K.
- Examples from the small-df group (Excel vs ours):
  - probes-numeric/t2220-f-dist-sweep-pdf Calc!A100 `F.DIST(100,1,100,FALSE)`: 2.4992492282800346e-17 vs
    2.4992492282800472e-17
  - probes-numeric/t2220-f-dist-sweep-pdf Calc!A199 `F.DIST(10,2,100,FALSE)`: 9.157068259764352e-5 vs 9.15706825976436e-5
  - probes-numeric/t2586-r4-f-small-df-a Calc!B5572 `F.DIST(A5572,100,1,FALSE)` (x 0.01658549389000001):
    8.355209916453504e-9 vs 8.355209916453491e-9
- Rule and research: docs/excel-semantics/statistics.md "F density is not the beta density, R's df or a GAMMALN
  formula (T-2513)" with its T-2586 and T-3034 updates; docs/research/t2464/r4-f-small-df (the e-table and bracket),
  r5-f-bigconst (`bcmodel.py`), r5-f-final (RESULTS.md section 4). Engine: `f_density.rs` (`BINET_TABLE`, the offset
  table C0 5.33 and C_BIG -0.42); code must not read docs/ (decision 0066). If T-2586 publishes a model for this
  constant first, port it instead of fitting.

### Cells owned
- probes-numeric/t2220-f-dist-sweep-pdf (6): A100, A199, A278, A299, A911, A921
- probes-numeric/t2586-r4-f-brcomp-hold (21): B257, B259, B260, B261, B262, B265, B267, B268, B270, B271, B272, B273,
  B275, B276, B277, B278, B279, B283, B284, B285, B286
- probes-numeric/t2586-r4-f-small-df-a (37): B5572, B5576, B5579, B5580, B5581, B5582, B5584, B5585, B5587, B5588,
  B5589, B5591, B5592, B5593, B5594, B5595, B5596, B5597, B5697, B5698, B5699, B5701, B5702, B5703, B5704, B5705,
  B5706, B5707, B5708, B5709, B5720, B5721, B5722, B5723, B5725, B5726, B5727
- probes-numeric/t2586-r4-f-small-df-hold (24): B193, B195, B196, B197, B198, B199, B200, B203, B204, B206, B207,
  B210, B213, B216, B219, B225, B226, B227, B228, B229, B230, B232, B239, B240
- probes-numeric/t2586-r5-f-edges-hold (18): B968, B969, B970, B971, B972, B973, B974, B975, B978, B979, B980, B981,
  B982, B983, B985, B986, B987, B988
- probes-numeric/t2586-r5-f-final-hold (34): B233, B234, B235, B236, B237, B238, B239, B240, B244, B245, B248, B1249,
  B1250, B1251, B1252, B1253, B1254, B1256, B1493, B1494, B1495, B1496, B1498, B1499, B1500, B1512, B1557, B1558,
  B1559, B1560, B1561, B1562, B1563, B1564
- probes-numeric/t2586-r5-f-smallr-a (1): B3448
- probes-numeric/t2586-verify-r5-f-bigconst-hold (3): B253, B254, B508
- probes-numeric/t2586-verify-r5-f-edges-hold (52): B97, B104, B125, B169, B183, B184, B185, B186, B187, B188, B189,
  B190, B191, B192, B193, B194, B231, B232, B233, B234, B235, B236, B237, B238, B240, B241, B242, B316, B317, B318,
  B319, B320, B321, B322, B323, B324, B325, B326, B340, B341, B342, B343, B344, B345, B346, B347, B348, B349, B350,
  D97, D104, D125
- probes-numeric/t2586-verify-r5-f-final-hold (24): B39, B40, B41, B42, B43, B141, B142, B143, B145, B146, B147, B148,
  B149, B150, B151, B152, B153, B159, B211, B212, B213, B214, B215, B216

## Acceptance criteria
- [ ] The fitted K (small-df delta as a function of the partner df, and the large-df constant if the four cells move
      with it) is chosen on some of the shapes above and scored on the others (held out by shape), with both counts
      in the Outcome (decision 0056: two attempts, then time-box and name what is left).
- [ ] `make eval`: at least half of the 220 cells above are within half a unit of Excel's 15th significant digit
      (`python3 scripts/exit_owners.py --all-off` no longer lists them), no F.DIST density cell at |P| < 4 elsewhere
      becomes visibly off, no new known-failure line appears in any eval/baseline file (`make eval-baseline`), and
      `make eval-check` passes.
- [ ] Unit tests pin bit-exact or 15-digit rows from at least three of the shapes above.
- [ ] statistics.md's F density entry records the fitted constant and its counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The power term at |P| >= 4 (T-3173); the F cdf (T-2508, T-2510, T-2219); huge df (T-2642).
- Bit-level residue left within 15 digits: a `defer: phase-1b` follow-up.

## Visible-parity stage
Decision 0114 `visible`: Excel's density is not the correctly rounded one here (M-final, Excel's own chain, already
beats the correctly rounded density on these regions), so the fix fits Excel's constant, never a more accurate one.
Target: the 220 cells within half a unit of Excel's 15th significant digit; whatever the held-out fit cannot reach is
named in the Outcome.
