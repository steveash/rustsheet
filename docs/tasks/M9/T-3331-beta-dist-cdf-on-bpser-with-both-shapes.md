---
id: T-3331
title: "BETA.DIST cdf on bpser with both shapes in (1, 8): BETA.DIST(0.1,2.5,5,TRUE) is 8 ulp below Excel, the last visibly-off cell the ported w3 model misses"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
defer: phase-1b
---

## Goal
`BETA.DIST(0.1,2.5,5,TRUE)` is within half a unit of Excel's 15th significant digit. Otherwise the Outcome records why
the bpser model of docs/excel-semantics/statistics.md ("Incomplete beta bpser", T-2464) cannot reach it at this point,
with the forms tried.

## Context
- Decision 0188 R5 (owner's 2026-10-31 rulings): parked as Phase 1b. Every cell this task owns is a BETA.DIST or
  F.DIST density cell or a dense probes-numeric sweep cell, within Gate A's envelope (most agree with Excel to 12 or
  more digits, and Excel and the engine are both close to the math), so R1(c) counts it as Phase 1b evidence. A
  point that Gate A's tables show outside the envelope gets its own task without the key.
Gap triage 2026-10-31 (`make eval` on 1171772e, `python3 scripts/exit_owners.py --all-off`, excel-graph). This is
the only visibly-off BETA.DIST cell no open task owns. BETA.DIST is below the M9 Phase 1 rate, so the cell counts in
the M9 exit row as `rate:BETA.DIST`. T-3249's Outcome noted it as predating that task; T-3007 (done) routed the case to
its bpser item but did not reach it.
- probes-numeric/m9-beta-dist-a2p5.case.json (1): Calc!A31

Also owned (gap triage 2026-10-30, moved here from the duplicate T-3252 of #2427; same bpser path,
last digits only, not required by the criteria below): Excel is within a unit of ours on each against a
60-digit value, so they follow A31's fix of the bpser sum (its term count, stop or stored z).
- probes-numeric/m9-beta-dist-a2p5.case.json (5): Calc!A3, A5, A7, A9, A37
- probes-numeric/m9-beta-dist-a2.case.json (5): Calc!A1, A3, A5, A37, A49
- probes-numeric/m9-legacy-betadist.case.json (2): Calc!A17, A18

`BETA.DIST(0.1,2.5,5,TRUE)`: Excel 0.05533381860850995, ours 0.055333818608509895. Ours is 8 ulp below Excel. The
true value (mpmath, 50 digits, gap triage) is 0.0553338186085098323, about 17 ulp below Excel and 9 below ours, so
Excel is the less accurate side.

Branch: a0 = 2.5, b0 = 5, b0·x0 = 0.5 <= 0.7, so bratio takes bpser with both shapes above 1 and b0 < 8. That is the
w3 forward series times EXP(fl(a LN x) - L) with L = the extended GAMMALN sum
(`functions/statistical/bratio/bpser.rs`). docs/research/t2464/r2-impact/offcells_pre.tsv (row
`BETA.DIST ... Calc!A31`) lists the T-2464 plan model's value for this cell: 0.055333818608509895, today's value. So
the model shipped as planned and this cell is the model's own miss, not a porting slip. That model is 4,602 of 4,602
within 15 digits on the 1 < b0 < 8 sub-branch of t2464-r2-bpser-small-shapes.

Leads, all numeric fitting:
- L. statistics.md says the L at 1 < b0 < 8 loses to r2-bratio's form on the older cells (3,365 against 4,038 of
  7,455) and that Excel's GAMMALN at the shape arguments matters. Check `GAMMALN(2.5)`, `GAMMALN(5)` and
  `GAMMALN(7.5)` against Excel's (t2464-r2-bpser-small-gl, or an excel-probe of the three) and score r2-bratio's L
  here.
- The series stop and the order of the c/(a+n) sum: at x = 0.1 the series is short, so a one-term difference shows.
- If this one point does not decide between forms, add an excel-probe sweep at a = 2.5 with b from 1.5 to 7.5 and
  x = 0.02 to 0.14 (b·x <= 0.7) before fitting (excel-probe skill). Do not fit blind.

## Acceptance criteria
- [ ] m9-beta-dist-a2p5 Calc!A31 is within half a unit of Excel's 15th significant digit. If not, the Outcome lists
      each form tried with its score on this cell and on t2464-r2-bpser-small-shapes' 1 < b0 < 8 cells (decision
      0056: two attempts, then time-box).
- [ ] No cell of t2464-r2-bpser-small-shapes, -runs, -gl or m9-beta-dist-a2p5 goes from within 15 digits to visibly
      off, and the bit-exact count on the 1 < b0 < 8 sub-branch does not drop. No new known-failure line in any
      eval/baseline file (`make eval-baseline`).
- [ ] statistics.md's "Incomplete beta bpser" entry records the outcome for this cell.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083/0114: the owned cell is visibly off, so this task is in the campaign. Excel is further from the true value
than the engine, so the stage is `visible`: reproduce Excel's bpser, never a more accurate kernel. The target is the
cell within half a unit of the 15th significant digit of Excel's value; bit-level misses left then go to a
`defer: phase-1b` follow-up.

## Out of scope
- bpser at b0 >= 8 (Excel's algdiv, T-3007's open item) and the density (T-3220, T-3244).
- bgrat, bfrac and L140 branches.
