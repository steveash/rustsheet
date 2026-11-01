---
id: T-3568
title: "Incomplete beta: n > 1 bup terms in bup+bgrat, bup's arithmetic and the a0 < 1e-15 band"
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
bup's terms and arithmetic inside L140's bup+bgrat are modelled, for n > 1 and for the F, BINOM and NEGBINOM callers.

## Context
- Split from T-3468 (size; one task per open item), itself split from T-3383 and T-2512. T-3383 pinned the density's
  lnB at min < 1 < 8 < max on 156 of 160 shapes (docs/research/t2464/r4-algdiv/RESULTS.md). Work in
  docs/research/t2464/ in a new `r5-*` directory, importing r2-bratio's `bratio_excel.py`, r2-tdist's `xlk.py`,
  r3-bgrat's `data.py`/`bscore.py` and r4-algdiv's `scan.py`. Method: docs/research/excel-kernel-playbook.md.
- Rules: positive controls before trusting a method; a real hold-out; record negatives (decision 0056); truncate
  formula literals to 15 digits as Excel does.
- **n > 1 bup terms** (2 <= b < 40, a > 15, x > 0.7) and the F, BINOM and NEGBINOM callers of L140 bup+bgrat: probe
  them and check that RN(W + D) holds, with W the n-term bup on Excel's density in bup's orientation.
- **bup's arithmetic in bup+bgrat** (T n <= 30: 356 of 1,344 bit-exact) and the a0 < 1e-15 band at
  0.04 <= y < 0.25 (Excel 1e-14 to 3.8e-13 relative off the true value, a problem in bgrat's incomplete-gamma ratio).

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
