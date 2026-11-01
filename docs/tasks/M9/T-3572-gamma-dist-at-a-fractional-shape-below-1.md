---
id: T-3572
title: "GAMMA.DIST at a fractional shape below 1 near P = 1: record or route the t2512-r3-bgrat-qgamma column A misses"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: accuracy
---

## Goal
The engine's GAMMA.DIST misses at a fractional shape below 1 near P = 1 are recorded in statistics.md and owned by a task.

## Context
- Split from T-3468 (size; one task per open item), itself split from T-3383 and T-2512. T-3383 pinned the density's
  lnB at min < 1 < 8 < max on 156 of 160 shapes (docs/research/t2464/r4-algdiv/RESULTS.md). Work in
  docs/research/t2464/ in a new `r5-*` directory, importing r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py`,
  r3-bgrat's `data.py`/`bscore.py` and r4-algdiv's `scan.py`. Method: docs/research/excel-kernel-playbook.md.
- Rules: positive controls before trusting a method; a real hold-out; record negatives (decision 0056); truncate
  formula literals to 15 digits as Excel does.
- **Excel's GAMMA.DIST at fractional b < 1 near P = 1**: probe t2512-r3-bgrat-qgamma column A is 1,268 of 4,980
  cells off in the engine, mostly 1-2 ulp, 117 at the 15th digit. Record it in the incomplete-gamma entries, or file
  an engine task if it is not one the existing incomplete-gamma tasks own.

## Acceptance criteria
- [ ] statistics.md's incomplete-gamma entries record the column A misses (counts by ulp distance, the 15th-digit cells).
- [ ] The cells are owned: an existing incomplete-gamma task's criteria are amended to name them, or a new engine task is
      filed with the cell list.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-3382 ports the L140 one-term composition).
- The other items split from T-3468: T-3567 (bgrat increment, F at df 1), T-3568 (bup), T-3569 (algdiv), T-3570
  (bpser's lnGamma and b < 1 ceiling), T-3571 (bfrac), T-3572 (GAMMA.DIST near P = 1); one each.
