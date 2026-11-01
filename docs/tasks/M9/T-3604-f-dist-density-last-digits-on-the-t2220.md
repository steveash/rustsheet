---
id: T-3604
title: "F.DIST density last digits on the t2220 d2 = 1 sweep and T-3478's wide sweep: 49 cells 5 to 23 ulp off Excel that no task names, scored on T-3173's and T-3174's models (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-3173, T-3174]
defer: phase-1b
---

## Goal
Once T-3173 (Excel's power term c·LN x where |c·LN x| >= 4) and T-3174 (the constant K where it is below 4) land, the
49 F.DIST density cells below are within 4 ulp of Excel's value, or the Outcome shows per cell why those models do
not reach them. The 252 below-floor density cells of the same two cases move no further from Excel.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 2f6dbaff1, excel-graph). An ownership check over every failing
  Excel cell (`scripts/exit_owners.py`'s `owned` rule, open tasks of origin/main) found 110 `F.DIST(x,d1,1,FALSE)`
  cells of the T-2220 d2 = 1 sweep and 191 density cells of T-3478's wide sweep (both listed under "Cells owned")
  that no open task names. None is visibly off: every one is within half a unit of Excel's 15th significant digit.
  Decision 0188 R5 makes this Phase 1b, hence `defer: phase-1b`.
- Already owned elsewhere, and out of scope here: T-2790 lists the d2-1 density cells where d1·x is large against d2;
  T-3592 (main) and T-3595 (open PR #2676) list the 48 visibly-off t3478 cells; open PR #2602's T-3474 lists 39
  visibly-off d2-1 cells. The d2-1 cdf and F.DIST.RT cells are T-2221's (its criterion on every T-2220 sweep cell).
- **Reproducibility floor (decision 0188 R4.4).** 252 of the 301 cells (84 d2-1, 168 t3478) are 1 to 4 ulp from
  Excel. They are not targets, only the no-regression set. The 49 targets are 5 to 23 ulp from Excel.
- **Region (M-final's, docs/research/t2464/r5-f-final/final_score.py `region`).** P = c·LN x with c = d1/2 when
  fl(d1·x) <= d2 and -d2/2 otherwise. 48 of the 49 targets have |P| >= 4 (T-3173's model): all 26 d2-1 targets (d1 3
  to 50 against d2 = 1 at x from 1E-6 to 0.09) and 22 of the t3478 targets. One, t3478 A183, has |P| < 4 (T-3174's
  constant).
- Excel is not the accurate side. Against the 40-digit density (mpmath, on the double arguments), Excel is closer
  than ours on 12 of the 26 d2-1 targets and 8 of the 23 t3478 targets. Both are often 10 to 50 ulp off in the same
  direction, as the shared power term predicts. Excel / ours, with the signed ulps of each from the exact value:
  - d2-1 A461 `F.DIST(1.74E-5,10,1,FALSE)`: 1.1268133870123145E-14 / 1.1268133870123108E-14 (-13.1, -36.1)
  - d2-1 A368 `F.DIST(6.72E-6,7,1,FALSE)`: 1.0818940027541965E-10 / 1.0818940027541986E-10 (-47.9, -31.9)
  - d2-1 A380 `F.DIST(0.000304,7,1,FALSE)`: 1.476843302467299E-06 / 1.4768433024672957E-06 (+20.3, +5.3)
  - t3478 A89 `F.DIST(0.00160207104951305,14,34758,FALSE)`: 1.9146825563182946E-14 / 1.9146825563182984E-14
    (+2.0, +14.0)
  - t3478 A293 `F.DIST(4.56777921700985,9,96129284,FALSE)`: 1.802416605318493E-05 / 1.802416605318497E-05
    (-10.4, +1.6)
- Rule and research: statistics.md "F density is not the beta density, R's df or a GAMMALN formula (T-2513)" with its
  T-2586 and T-3034 updates, and "F density's small-shape constant past its fit set" (T-3478). Code: `excel_f_density`
  in crates/rustsheet-engine/src/functions/statistical/f_density.rs. Engine code must not read docs/ (decision 0066).
- This task scores and, if needed, extends those models. It does not fit these 49 cells on their own: the t2220 and
  t3478 sweeps are also the models' hold-out evidence, and fitting their bits here would leak.
- Cells owned:
  - probes-numeric/t2220-f-dist-sweep-d2-1.case.json (26 targets): Calc!A101, A182, A200, A275, A281, A284, A296, A368, A371, A374, A377, A380, A383, A386, A452, A461, A467, A482, A488, A542, A551, A560, A563, A569, A572, A653
  - probes-numeric/t3478-f-density-r4-wide.case.json (23 targets): Calc!A19, A58, A65:A66, A68:A69, A74, A76, A79, A87, A89, A92, A115, A130, A164, A167, A183, A243, A248, A266, A269, A281, A293
  - probes-numeric/t2220-f-dist-sweep-d2-1.case.json (84, no-regression set, 1 to 4 ulp): Calc!A5, A8, A11, A14, A17, A20, A23, A26, A29, A32, A35, A44, A92, A95, A98, A104, A107, A110, A113, A116, A119, A128, A134, A137, A140, A146, A158, A185, A194, A197, A203, A206, A209, A212, A215, A218, A221, A224, A287, A293, A299, A302, A308, A314, A317, A320, A323, A362, A365, A389, A398, A410, A413, A470, A476, A485, A491, A494, A497, A500, A578, A656, A668, A674, A677, A680, A710, A713, A716, A719, A758, A764, A776, A779, A782, A785, A788, A791, A794, A797, A800, A803, A806, A809
  - probes-numeric/t3478-f-density-r4-wide.case.json (168, no-regression set, 1 to 4 ulp): Calc!A1:A3, A6, A8:A12, A14:A15, A17, A20:A28, A30:A33, A35:A40, A42, A44:A49, A52, A55:A56, A61, A67, A70:A73, A77, A84, A86, A88, A93, A97, A100:A103, A105:A106, A108, A111, A113, A116:A118, A121, A123, A126, A132:A133, A135, A137:A143, A145, A147:A152, A159:A160, A165, A169:A173, A175:A177, A181:A182, A186, A191:A194, A196, A198:A201, A203:A204, A206:A211, A213:A215, A217:A218, A220:A221, A225:A238, A240:A242, A244:A245, A247, A250, A253, A255:A256, A258:A260, A265, A270:A271, A276:A277, A279, A283, A285, A287:A291, A295:A299

## Acceptance criteria
- [ ] Each of the 49 targets is within 4 ulp of Excel's value (`make eval-show CASE=...` on
      the two cases under "Cells owned"), or the
      Outcome lists it with Excel, ours, the 50-digit density and the reason the models do not reach it.
- [ ] None of the 252 no-regression cells moves beyond 4 ulp of Excel, and the bit-exact and within-15-digit counts of
      both cases (810 and 300 cells) do not fall. Counts before and after are in the Outcome.
- [ ] No new known-failure line in any eval/baseline/*.txt (`make eval-baseline` refuses regressions). Both cases are
      probes-numeric, which has no baseline, so the report.json counts above are the measure.
- [ ] Any fitted change meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188), each shown in the
      Outcome:
      - hold-out counts (bit-exact, within 4 ulp, within 15 digits) for the chosen model and for the correctly rounded
        density, from a split stated before scoring;
      - a fresh sweep after the freeze (a new probes-numeric case over d2 = 1 to 2 and min(d) 11 to 15, x log-uniform);
      - the Gate A envelope result on that sweep, every point outside it listed;
      - the `FITTED(T-3604, n_cells, range)` tag on each fitted piece, applied only inside its range;
      - the reproducibility floor: no step fits differences of 4 ulp or less;
      - no keep-matched branch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells named under Context as owned by T-2790, T-3592, T-3595, T-3474 and T-2221.
- The F cdf and F.DIST.RT (T-2221), and the density models themselves (T-3173, T-3174).
