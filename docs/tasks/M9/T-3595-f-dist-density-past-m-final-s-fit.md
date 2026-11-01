---
id: T-3595
title: "F.DIST density past M-final's fit set: the 48 visibly-off t3478-f-density-r4-wide cells (min df 11 to 15, max df to 1E+10)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3173, T-3174]
campaign: visible
defer: phase-1b
---

## Goal
After T-3173's power-term model and T-3174's constant K land, the 48 F.DIST density cells of
probes-numeric/t3478-f-density-r4-wide are within half a unit of the 15th digit of Excel's value. Or the Outcome shows,
per cell, why the shapes past M-final's fit set need more than those models.

## Context
- Gap triage 2026-11-01 (`make eval` on main 408444f20, excel-graph). T-3478 (done) drew this 300-point sweep after
  the F density's small-shape constants were frozen, to check them past their fit set. It kept the constants:
  statistics.md "F density's small-shape constant past its fit set" has the per-stratum table. It filed no owner for
  the cells still visibly off against Excel, so `exit_owners.py --milestone M9` lists them under `rate:F.DIST` and no
  open task names the case.
- Parked as Phase 1b (decision 0188 R5, as for its siblings T-3158, T-3173 and T-3174). All 300 points are within
  1e-9 of the 50-digit density (the case's `.math.tsv`; statistics.md), so R1(c) counts these cells as Phase 1b
  evidence. A point that Gate A's tables show outside the envelope gets its own task without the key.
- Classified with M-final's regions (side 1 when fl(d1·x) <= d2, P = c·LN x with c = d1/2 on side 1 and -d2/2 on
  side 2; docs/research/t2464/r5-f-final/final_score.py `region`):
  - |P| >= 4, the power term, T-3173's model (45 cells). 37 have min(d) 5 to 15 ("mid") and 8 have min(d) < 5.
    The largest misses against Excel are at a huge partner df: A274 `F.DIST(613.315085228254,2,800347830,FALSE)`
    Excel 4.373672555798298e-267, ours 4.373672555798766e-267 (1.1e-13 relative, P 6.4); A268
    `F.DIST(157.268096838926,7,1694894086,FALSE)` 6.637536879737717e-233 vs 6.637536879737135e-233 (8.8e-14); A98
    `F.DIST(0.00193736953873672,82,15,FALSE)` 3.815901356089233e-70 vs 3.815901356089266e-70 (P -256).
  - |P| < 4, the constant K at a df of 1 or 2 against a large partner, T-3174's model (3 cells): A162
    `F.DIST(0.0281038920010623,52,1,FALSE)` 8.428168215063469e-05 vs 8.428168215063486e-05; A174 (93, 2); A273
    (861, 1).
- Rule and research: statistics.md "F density is not the beta density, R's df or a GAMMALN formula (T-2513)" (T-2586
  and T-3034 updates) and "F density's small-shape constant past its fit set". Code: `excel_f_density` in
  crates/rustsheet-engine/src/functions/statistical/f_density.rs. Engine code must not read docs/ (decision 0066).
- Cells owned:
  - probes-numeric/t3478-f-density-r4-wide.case.json (48): Calc!A41, A50, A53, A54, A59, A60, A62, A63, A64, A75,
    A78, A81, A82, A85, A90, A94, A95, A96, A98, A99, A154, A156, A157, A161, A162, A166, A174, A178, A184, A187,
    A188, A189, A197, A249, A252, A257, A261, A262, A264, A267, A268, A272, A273, A274, A275, A278, A282, A300

## Acceptance criteria
- [ ] Re-measured on main once T-3173 and T-3174 are done: the count of owned cells still visibly off, split by the
      regions above, is in the first Outcome line.
- [ ] Each owned cell is within half a unit of the 15th significant digit of Excel's value, or is named in the
      Outcome with its region and its error against Excel and against the math. Cells that need a model past the
      fit sets of T-3173/T-3174 move to a follow-up with the reason.
- [ ] Any constant or form refitted here follows decision 0188 R4 (reviewer-excel-fidelity check 7), shown in the
      Outcome: hold-out counts against the correctly rounded density (R4.1); a fresh sweep oracle-filled after the
      freeze (R4.2); its Gate A envelope result, 0 points outside (R2); a `FITTED(T-3595, n_cells, range)` tag with
      the fallback to accurate math outside the range (R4.3); no step below the reproducibility floor (R4.4); no
      keep-matched branch (decisions 0056, 0082).
- [ ] No gated cell regresses (`make eval-baseline`); `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 and 0114 (`visible`): done when every owned cell is within half a unit of the 15th significant digit of
Excel's value. Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Out of scope
- The power term and K themselves (T-3173, T-3174, research T-2586). The t2220/t2513/t2586 sweep cells (T-3158,
  T-3173, T-3174). F.DIST at a tiny x (T-3479, T-3527).
