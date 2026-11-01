---
id: T-3730
title: "T.TEST and TTEST: the 9 visibly-off gen-m9 cells only time-boxed T-0969 owns stray 5 to 19 ulp from the rounded-y rule that gives 1 to 8"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The 9 gen-m9 T.TEST and TTEST cells below are within half a unit of Excel's 15th significant digit. Today their only
open owner is T-0969, which is blocked and time-boxed (decision 0056), so the campaign queue never reaches them. The
fix should make our p-value follow the rule statistics.md already documents, "T.TEST rounds y = df/(df+t^2) to a
double before the tail", or else record why it cannot.

## Context
- Gap triage 2026-11-01h (`make eval` on origin/main d2ddb42a7, excel-graph). This was a status split of
  `scripts/exit_owners.py`'s `owned` rule over every `--all-off` cell: open, undeferred owners, minus the blocked
  ones. It leaves these cells with T-0969 as their only owner. T-0969 lists them in its "Owned gen-m9 cells
  (T-2256 triage)" block. T-3079 routes T-0969's probes-numeric sweeps but not these gen-m9 cells. T-3254 (deferred)
  owns only T.TEST cells that are already within 15 digits, and T-2930 owns T.TEST's complement probe. All three are
  named here so the split is explicit (T-3229).
- The cells are known-failure lines of eval/baseline/gen-m9.txt:
  - gen-m9/928d08e3e1eecb5f.case.json (1): Calc!A393
  - gen-m9/9bc247733a1ca9fa.case.json (2): Calc!A139, A254
  - gen-m9/afc00c8b70e78baf.case.json (1): Calc!A238
  - gen-m9/09b47629f54b9817.case.json (1): Calc!A144
  - gen-m9/c7664e494e418395.case.json (1): Calc!A219
  - gen-m9/ce312817dc9c7f0b.case.json (1): Calc!A228
  - gen-m9/f82473275380882e.case.json (1): Calc!A308
  - gen-m9/d5fd9e05e9e6f466.case.json (1): Calc!A246
- Excel's value, ours, and a throwaway model, in ulp from Excel. The model computes the statistic in plain doubles
  (sequential sums for the means, the sum of squared deviations over n - 1 for the variances, pooled for type 2,
  Welch for type 3), then y = fl(df/fl(df + fl(t·t))), then mpmath's exact `betainc(df/2, 1/2, 0, y)` times tails/2:
  | cell | formula | Excel | ours | model |
  |---|---|---|---|---|
  | 928d08e3e1eecb5f A393 | `T.TEST(Data!B3:C11,Data!B4:E6,2,2)` | 0.7772031601919551 | -18 | -4 |
  | ce312817dc9c7f0b A228 | `TTEST(Data!A1:E1,Data!A1:D7,2,2)` | 0.609938634913881 | -19 | -8 |
  | 9bc247733a1ca9fa A254 | `T.TEST(Data!A6:A12,Data!A4:B12,2,2)` | 0.9999998773979659 | +10 | +1 |
  | f82473275380882e A308 | `TTEST(Data!A4:C4,Data!B4:C12,2,2)` | 0.6436799928845899 | +8 | +3 |
  | 9bc247733a1ca9fa A139 | `TTEST(Data!B9:E9,Data!A4:D7,1,3)` | 0.08451014632899068 | +7 | +3 |
  | 09b47629f54b9817 A144 | `TTEST({100000000.1,100000000.2,100000000.3},Data!C1:E10,2,2)` | 0.6683531037456522 | +5 | +2 |
  | c7664e494e418395 A219 | `TTEST(Data!B11:E12,Data!C6:E8,1,3)` | 0.09084506673248453 | -5 | -2 |
  | afc00c8b70e78baf A238 | `T.TEST(Data!B8:E12,Data!A8:B8,2,2)` | 0.725939456341534 | +5 | +1 |
  | d5fd9e05e9e6f466 A246 | `T.TEST(Data!A6:D12,{1,2},VLOOKUP(..),3)` | 0.1638753783816484 | +25 | not run (its tails argument is a VLOOKUP) |
  The documented rule beats our engine on all eight cells. So ours has drifted from its own rule, in the
  statistic, in the tail (`student_t::student_t_tail_at`, the pre-T-2509 path T.TEST still takes), or in both.
- Against 50-digit math (mpmath, the exact statistic and exact tail), Excel is 2e-16 to 2.1e-15 relative off, and so
  are we (up to 2.5e-15). Excel is not correctly rounded here, so the stage is `visible`, not `accuracy`.
- Code: `t_test_p` in crates/rustsheet-engine/src/functions/statistical/t_dist.rs (the `mean` and `sample_var`
  helpers, the pooled and Welch statistic, `t_square`) and `student_t_tail_at` in student_t.rs.
- Evidence beyond these cells: probes-numeric t1048-ttest and t1049-ttest (about 90 T.TEST/TTEST cells,
  excel-graph), ironcalc-calc STATISTICAL/T_TEST.xlsx, and the T.TEST cells of gen-m9 that pass today, which must
  keep passing. Those give enough points to rank candidates, so no new sweep is needed before the fit. R4.2's fresh
  sweep comes after the model is frozen.

## Acceptance criteria
- [ ] First a trace of the statistic and y for each listed cell: which step departs from the plain-double rule (the
      mean or variance summation, the pooled or Welch form, `t_square`, or the tail at y).
- [ ] The 9 known-failure lines above are gone from eval/baseline/gen-m9.txt after `make eval-baseline`,
      with no new line in any baseline. If some cells stay off, the Outcome lists each candidate with its counts and
      moves the rest to a follow-up (decision 0056 time box).
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), required for any fitted choice; a structural fix to
      the documented rule needs only the first and the fifth:
  - Hold-out counts: the fit lab's fixed split (sha256 of `rustsheet-fitlab-holdout/v1`) over every T.TEST/TTEST
    oracle cell of the suites, set before scoring. Train and test exact, within 4 ulp and within 15 digits are
    reported side by side; the candidate beats the correctly rounded one on test.
  - Fresh sweep: a new log-uniform T.TEST sweep (types 1 to 3, tails 1 and 2, n 2 to 50 per sample, magnitudes
    1e-3 to 1e9) is filled on excel-graph after the model is frozen, and its counts are recorded here before merge.
  - Gate A envelope: `rustsheet-eval accuracy` passes, and the Outcome gives T.TEST's Gate A result, or "no table".
  - Each fitted constant, cut or table carries `/// FITTED(T-3730, <n> cells, <range>)`, and outside that range the
    code falls back to accurate math.
  - Reproducibility floor: no fitting below decision 0112's floor (web against desktop up to 4 ulp).
  - No keep-matched branch: no special case that only reproduces the listed cells.
- [ ] docs/excel-semantics/statistics.md, in "T.TEST rounds y = df/(df+t^2) to a double before the tail", records the
      statistic's double form that matches, citing these cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T.TEST's complement, and routing it through the K kernel (T-2930). T.DIST.2T and T.DIST.RT cells (T-2930,
  T-0969). Cells already within 15 digits (T-3254, deferred). TTEST over PIVOTBY slices (T-2993).

## Visible-parity stage
Target (decision 0114): the 9 listed cells are within half a unit of Excel's 15th significant digit. Their bits are
Phase 1b: whatever is still bit-level when the target is met goes to T-3254.
