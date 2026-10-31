---
id: T-3479
title: "BETA.DIST cdf on L140 bup+bgrat at a below 1, b = 2.5 and x in [0.29, 0.3): Excel is within 3E-16 of the math, ours 1E-15 (9 t2464-r2-bratio-x29 cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BETA.DIST's cdf at a first shape below 1, b = 2.5 and x just below 0.3 is within half a unit of the 15th significant
digit of Excel's value on the 9 t2464-r2-bratio-x29 cells still visibly off, through a more accurate evaluation there,
not a fit to Excel's bits.

## Context
Gap triage 2026-10-31 (`make eval` on main b9558688e). T-2510 listed these 9 as "BETA.DIST cdf, BupBgrat 9" and handed
the bgrat-family cells still off to T-2512 (done); no open task names them now. Scored in triage against 50-digit mpmath
(`betainc`, regularized): Excel is 4E-17 to 3E-16 relative off the math, ours 6.5E-16 to 1.1E-15, so Excel is close to
correctly rounded and the engine is not (decision 0114 `accuracy`). Example: A1574 `BETA.DIST(0.2933,0.75,2.5,TRUE)`,
Excel 0.6824095513316164, ours 0.682409551331617.
- probes-numeric/t2464-r2-bratio-x29.case.json (9): Calc!A1551, A1574, A1597, A1609, A1616, A1637, A2327, A2393, A2408

TOMS 708 sends a0 = 0.75 or 0.9, b0 = 2.5, x0 in [0.29, 0.3) to L140 (bup, then bgrat); the port takes Excel's 0.3
switch (statistics.md "x0 >= 0.3, not R's 0.29": Excel matches the 0.3 dispatch on 284 of the 434 differing cells of
this case, the 0.29 dispatch on 87, neither on 63). Start by checking which of the three classes these 9 fall in. If
Excel's values equal the 0.29 dispatch (w = 1 - bpser(b0, a0, y0)), that path is the accurate one here and the question
is the selector. If they match neither, the bup sum or bgrat's series loses the digits: find which term, as
T-3382 does for 1 < b < 2 on the same branch (`bratio` in functions/statistical/, `beta_dist_lower_tail`).

## Acceptance criteria
- [ ] The 9 cells above are within half a unit of the 15th significant digit of Excel's value: after `make eval` none of
      them is among the visibly-off cells (`attributed_off`) of eval/runs/latest/report.json (probes-numeric is
      ungated, so it has no baseline file). No line is added to any eval/baseline/ file.
- [ ] Which dispatch class each cell is in is recorded in statistics.md under the x0 >= 0.3 entry.
- [ ] The change makes the kernel more accurate. It has no selector tuned to these cells: any new branch condition
      is justified by the math (error bound) or by the 434-cell x29 evidence and named in the code comment.
- [ ] No probes-numeric or gated cell goes from within 15 digits to visibly off (per-case deltas in the Outcome).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Bit parity on the x29 cells already within 15 digits (Phase 1b, decision 0188 R5).
- L140 at 1 < b < 2 (T-3382).

## Visible-parity stage
Decisions 0083 and 0114 (`accuracy`): done when the 9 owned cells are within half a unit of the 15th significant digit of
Excel's value. Time-boxed by decision 0056.
