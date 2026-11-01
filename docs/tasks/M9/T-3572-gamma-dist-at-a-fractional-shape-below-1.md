---
id: T-3572
title: "GAMMA.DIST at a fractional shape below 1 near P = 1: record or route the t2512-r3-bgrat-qgamma column A misses"
milestone: M9
lane: research
status: done
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

## Outcome
Counted the engine's column A misses on t2512-r3-bgrat-qgamma (docs/research/t2464/r5-gamma-p1/ulps.py): 1,268 of
4,979 numeric cells, every one exactly 1 ulp (none at 2 or more), 117 off at the 15th digit. Recorded in
statistics.md with counts by shape; cell list in r5-gamma-p1/misses.tsv. Owned by T-3224 (owned-cells list amended).
No engine change; eval delta none. Negative: no kernel was tried here (out of scope).
Follow-up for the owner: T-3224 is `defer: phase-1b` on the ground that none of its cells is visibly off; its own rule
says to give it a `campaign:` stage once one is. 117 of the cells added here are off at the 15th digit, so the
steward should decide that.
