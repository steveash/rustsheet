---
id: T-3592
title: "Owners for 121 unowned last-digit stat cells inside Gate A: BETA.DIST (39), F.DIST.RT (36), the T-3478 F density sweep (45) and CONFIDENCE.T (1)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Each cell listed below is within half a unit of the 15th significant digit of Excel's value. A cell can also be moved
to a narrower task that names its kernel branch, or listed in the Outcome with Excel, ours and the 50-digit math and
the reason Excel's digits cannot be derived.

## Context
- Gap triage 2026-11-01 (`make eval` on origin/main 408444f20, `scripts/exit_owners.py --all-off`): these cells are
  visibly off, and no open task names them. All are last-digit work. The triage computed the math at 50 digits on the
  double inputs. Excel and ours are within 4.3e-14 relative of it on every BETA.DIST and F.DIST.RT cell. T-3478
  measured the F density sweep inside 1e-9. Ours is the correctly rounded value on the CONFIDENCE.T cell.
  - Decision 0188 R5 makes this Phase 1b, hence `defer: phase-1b`.
  - They count in the R1(c) rows today only because BETA.DIST, F.DIST.RT and CONFIDENCE.T have no Gate A definition
    (T-3529 adds BETA.DIST) and the F density sweep has no `.math.json` (T-3587).
- Groups by root cause:
  - **BETA.DIST through the incomplete beta and the Loader density (39).** These are bratio branches and the density
    at a large shape: a = 100 with b from 1E-4 to 100, a = 5 or 10 against b up to 100, and a just below 1 against
    b = 2.5 or 5 (the w2f unit-shape cells, a = 1 - 2^-26 and its neighbours). See statistics.md "Incomplete beta",
    T-3443 (basym at two large shapes), T-3323 (bfrac bit level) and T-3035 (density for min(a, b) < 1). Move a cell
    to one of those if its branch matches theirs.
  - **F.DIST.RT on T-2219's half-integer kernel and bratio (36).** t2220's sweeps at d1, d2 up to 100. Examples:
    `F.DIST.RT(1,100,100)`, Excel 0.49999999999999967 and ours 0.5000000000000002 (the math is 1/2 exactly);
    `F.DIST.RT(1,100,50)`, Excel 0.5108794921920699 and ours ...683. T-3199 owns the same family's cells where Excel is
    5-50 ulp off the math.
  - **F density past the T-3478 fit set (45).** statistics.md "F density's small-shape constant past its fit set";
    T-3158, T-3173 and T-3174 hold the model work.
  - **CONFIDENCE.T at alpha near 1, df 2 (1).** `CONFIDENCE.T(0.999999999989522,1,3)` is Excel 8.555211852081089E-12
    and ours 8.555211852081079E-12, where the math is 8.5552118520810793E-12 (Excel is 1.1e-15 off). It comes from
    T.INV.2T(p,2) near p = 1, the T-2724 centre path.
- Cells:
  - probes-numeric/m9-beta-dist-a10.case.json (5): Calc!A20, A31, A32, A48, A54
  - probes-numeric/m9-beta-dist-a100.case.json (14): Calc!A5, A20, A25, A26, A28, A30, A32, A34, A38, A43, A44, A46, A48, A51
  - probes-numeric/m9-beta-dist-a5.case.json (5): Calc!A5, A31, A42, A52, A54
  - probes-numeric/t2464-r2-bratio-x29.case.json (9): Calc!A1551, A1574, A1597, A1609, A1616, A1637, A2327, A2393, A2408
  - probes-numeric/t2464-w2f-unit-shape.case.json (6): Calc!A211, A407, A409, A411, A421, A423
  - probes-numeric/t1538-beta-dist-at-excel-inv.case.json (1): Calc!A15
  - probes-numeric/t2220-f-dist-sweep-d2-1.case.json (2): Calc!A318, A675
  - probes-numeric/t2220-f-dist-sweep-rt.case.json (33): Calc!A84, A97, A98, A399, A467, A504, A567, A579, A589, A594, A596, A669, A680, A697, A795, A800, A804, A868, A878, A879, A885, A890, A897, A900, A954, A960, A978, A984, A990, A994, A997, A999, A1000
  - probes-numeric/t3478-f-density-r4-wide.case.json (45): Calc!A53, A54, A59, A60, A62, A63, A64, A75, A78, A81, A82, A85, A90, A94, A95, A96, A98, A99, A154, A156, A161, A162, A166, A174, A178, A184, A187, A188, A189, A197, A249, A252, A257, A261, A262, A264, A267, A268, A272, A273, A274, A275, A278, A282, A300
  - probes/t3349-t-inv-2t-near-one.case.json (1): Calc!A5

## Acceptance criteria
- [ ] Each listed cell is within half a unit of Excel's 15th significant digit, or moved to a named narrower task
      (both files name each other), or listed in the Outcome with the three values and the reason.
- [ ] Any fitted change meets the R4 guards of reviewer-excel-fidelity check 7 (decision 0188):
      - hold-out counts stated before scoring;
      - a fresh sweep after the freeze;
      - the Gate A envelope result;
      - the `FITTED` tag with its range;
      - the reproducibility floor;
      - no keep-matched branch.
- [ ] No gated known-failure line is added (`make eval-baseline` refuses regressions), and the BETA.DIST and F.DIST
      fit sets of the tasks above keep their counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells T-3199, T-3527, T-3479 and T-2221 already list.
