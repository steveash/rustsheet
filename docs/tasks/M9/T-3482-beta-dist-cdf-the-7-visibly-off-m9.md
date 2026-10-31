---
id: T-3482
title: "BETA.DIST cdf: the 7 visibly-off m9-beta-dist cells at shapes 5 to 100 that T-2508, T-2510 and T-3007 left (bgrat, bfrac and basym branches)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
The 7 BETA.DIST cdf cells below are within half a unit of the 15th significant digit of Excel's value. Each one is
fixed on its own bratio branch: an accurate formulation where ours loses digits Excel keeps, and Excel's formulation
where Excel is the one off the math.

## Context
Gap triage 2026-10-31 (`make eval` on main b9558688e). T-2508, T-2510 and T-3007 (all done) each named these cases,
and no open task owns these cells now:
- probes-numeric/m9-beta-dist-a5.case.json (2): Calc!A5, A31
- probes-numeric/m9-beta-dist-a10.case.json (1): Calc!A31
- probes-numeric/m9-beta-dist-a100.case.json (4): Calc!A5, A25, A43, A51

Relative error against 50-digit mpmath `betainc` at the double x, from triage (ours / Excel):
| cell | formula | Excel | ours | ours off | Excel off |
|---|---|---|---|---|---|
| a5 A5 | `BETA.DIST(0.9,5,0.0001,TRUE)` | 5.905821819121897e-5 | 5.90582181912189e-5 | 5.0E-16 | 6.8E-16 |
| a5 A31 | `BETA.DIST(0.1,5,5,TRUE)` | 0.0008909200000000032 | 0.0008909200000000024 | 2.4E-15 | 3.3E-15 |
| a10 A31 | `BETA.DIST(0.1,10,5,TRUE)` | 6.840415000000027e-8 | 6.840415000000021e-8 | 2.5E-15 | 3.4E-15 |
| a100 A5 | `BETA.DIST(0.9,100,0.0001,TRUE)` | 2.4536931034611277e-10 | 2.4536931034611685e-10 | 1.5E-14 | 1.6E-15 |
| a100 A25 | `BETA.DIST(0.1,100,2.5,TRUE)` | 6.554404786093256e-98 | 6.554404786093248e-98 | 1.2E-14 | 1.3E-14 |
| a100 A43 | `BETA.DIST(0.1,100,50,TRUE)` | 4.060690684813406e-63 | 4.0606906848133605e-63 | 1.6E-14 | 4.9E-15 |
| a100 A51 | `BETA.DIST(0.5,100,100,TRUE)` | 0.4999999999999999 | 0.5000000000000004 | 8.0E-16 | 2.0E-16 |

The triage reference used the exact double x with 1 - x exact. BETA.DIST's caller passes y = fl(1 - x)
(statistics.md "Caller arguments"), and at a shape of 100 the cdf's sensitivity to that rounding is about 1E-14. Redo
the reference on the (x, fl(1 - x)) pair before calling a cell accuracy or Excel's own: A25, where both are 1.2E-14
off, is probably that rounding. A100 A5 and A43 (ours 3 to 10 times further off than Excel) and A51 (the symmetric
point, where Excel is under 1/2 and ours over) look like our loss. Branch per cell: run `bratio::dispatch` on each
cell's oriented (a0, b0, x0, y0). For example, `(0.9, 100, 1E-4)` swaps to a0 = 1E-4, b0 = 100, x0 = 0.1, which is
bgrat since b0 > 15. Then compare with the branch owners' rules in statistics.md.

## Acceptance criteria
- [ ] Each cell's branch and its error against the (x, fl(1 - x)) reference are recorded in the Outcome.
- [ ] The 7 cells are within half a unit of the 15th significant digit of Excel's value: after `make eval` none of them
      is among the visibly-off cells (`attributed_off`) of eval/runs/latest/report.json. No line is added to any
      eval/baseline/ file. If a cell needs a fit that the cached evidence cannot support, it is split into a fit task
      that depends on a research-lane oracle sweep, and the Outcome names the new task.
- [ ] Any fitted or ranked choice meets reviewer-excel-fidelity check 7 (decision 0188 R4). That means hold-out counts
      side by side (R4.1), a fresh sweep filled after the freeze with its counts (R4.2), and the Gate A envelope result
      on it with every point outside listed (R2). Also a `FITTED(T-3482, <n> cells, <range>)` tag with an accurate
      fallback outside the range (R4.3), no target at or below the 4-ulp reproducibility floor (R4.4), and no
      keep-matched branch (decisions 0056, 0082).
- [ ] No probes-numeric or gated cell goes from within 15 digits to visibly off (per-case deltas in the Outcome).
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.DIST densities in these cases (Phase 1b, T-3220 and T-3244, decision 0188 R5).
- L140 bup+bgrat at a < 1 near x = 0.3 (T-3479) and shapes next to 1 (T-3481).

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when the 7 owned cells are within half a unit of the 15th significant digit of
Excel's value. Time-boxed by decision 0056.
