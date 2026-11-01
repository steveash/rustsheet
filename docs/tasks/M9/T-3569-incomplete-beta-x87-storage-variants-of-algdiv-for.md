---
id: T-3569
title: "Incomplete beta: x87 storage variants of algdiv for b0 >= 8 against the r4-algdiv pins, then bpser's L"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
algdiv for b0 >= 8 is modelled as Excel stores it, first on the density's pinned lnB, then in bpser's L.

## Context
- Split from T-3468 (size; one task per open item), itself split from T-3383 and T-2512. T-3383 pinned the density's
  lnB at min < 1 < 8 < max on 156 of 160 shapes (docs/research/t2464/r4-algdiv/RESULTS.md). Work in
  docs/research/t2464/ in a new `r5-*` directory, importing r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py`,
  r3-bgrat's `data.py`/`bscore.py` and r4-algdiv's `scan.py`. Method: docs/research/excel-kernel-playbook.md.
- Rules: positive controls before trusting a method; a real hold-out; record negatives (decision 0056); truncate
  formula literals to 15 digits as Excel does.
- **algdiv for b0 >= 8.** Score x87 storage variants of TOMS algdiv (r3-m1-residual's `algsearch.py` grid,
  `algdiv_v.py`) per shape against r4-algdiv's 156 pinned lnB values (`scan_a.json`, `scan_hold.json`; in-sample a
  only, hold-out once at the end), not per row. Then check the winner in bpser's L (the cdf; implied L -1.88, +1.65
  and +6.05 units on r2-bpser-small's runs). Rejected in r4-algdiv: the correctly rounded lnB (34 and 37 of 80
  shapes), fl(GL(a) + CR algdiv) (22, 35), TOMS-d (26, 30), Excel's GAMMALN difference past 8 in either
  association (0-1).
  - The pins are Excel for the web values, and their -4 to +3 ulp offsets are inside the web/desktop floor of up
    to 4 ulp (docs/research/oracle-calibration.md). Spot-check a winning model on desktop Excel before freezing it
    (M14 if only desktop can answer). Rescan (0.4, 8.6) wider: it pins at the scan's edge, j = -16, 3 of 50.
  - At a = 1 or b = 1 the density's lnB is -LN(the other shape) at any size, not algdiv (statistics.md "Beta density
    for min(a,b) <= 1").

## Acceptance criteria
- [ ] A RESULTS.md records a model with exact, within-4-ulp and within-15-digit counts on a held-out probe, or the
      negatives tried with their counts.
- [ ] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's T-2464 entries are updated with what is found.
- [ ] Engine follow-ups are filed for any solved piece, or the owning engine task's criteria are amended.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-3382 ports the L140 one-term composition).
- The other items split from T-3468: T-3567 (bgrat increment, F at df 1), T-3568 (bup), T-3569 (algdiv), T-3570
  (bpser's lnGamma and b < 1 ceiling), T-3571 (bfrac), T-3572 (GAMMA.DIST near P = 1); one each.
