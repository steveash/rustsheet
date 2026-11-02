---
id: T-3750
title: "Beta density algdiv lnB (min < 1 < 8 < max): desktop Excel cells for the r4-algdiv shapes"
milestone: M14
lane: research
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Desktop Excel values settle whether the density's lnB for min(a, b) < 1 < 8 < max(a, b) is the correctly rounded value or carries rounding noise of a few ulps.

## Context
T-3569 (docs/research/t2464/r5-algdiv/RESULTS.md) scored 65,792 x87 storage variants of TOMS algdiv per shape
against r4-algdiv's pinned lnB. None reproduced most shapes: at best 36 and 33 of 78, against 33 and 35 for the
correctly rounded lnB. The pins come from Excel for the web, and their -4 to +3 ulp spread is inside the web/desktop
floor (docs/research/oracle-calibration.md), so web pins cannot tell noise from model. Fill probes-numeric
t3383-r4-algdiv-a and -hold (or a subset of their shapes, 50 x each) on excel-desktop. Then pin per shape with
r4-algdiv's `scan.py` and re-score r5-algdiv's `counts.py` candidates. Also rescan (0.4, 8.6) wider than j = +-16
(its web pin sits at the edge).

## Acceptance criteria
- [ ] Desktop sidecars for the r4-algdiv shapes are committed, and per-shape pins are computed from them.
- [ ] statistics.md's T-2464 algdiv entry records whether the desktop pins equal the correctly rounded lnB or a
      candidate, with the counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (an engine task follows if a candidate reaches most shapes).

## Blocked because
Phase 4 (decision 0039): needs the excel-desktop backend. Unblock when `cd tools && uv run rustsheet-oracle status`
shows `excel-desktop OK`.
