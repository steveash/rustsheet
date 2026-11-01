---
id: T-3582
title: "Incomplete beta: the white noise in Excel's direct bgrat increment at b != 1/2 (Excel's own Q(b, z) inside bgrat)"
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
Excel's direct bgrat increment D at b != 1/2 is modelled to the bit, or the source of its noise is named with
evidence.

## Context
- Split from T-3567 (docs/research/t2464/r5-bgrat-fdf1/RESULTS.md section 1 item 4 and section 3). Along the
  split probe's runs of 1,024 consecutive doubles (t2512-r3-bgrat-split, (30.7, 0.7) and (40, 0.03)), Excel's D
  minus the exact I at its stored ln x is white noise of sd 5.7 and 8.5 units of 2^-53. Only the stored
  z = fl(-ν·ln x) shows. The prefactor's form, ln x by log1p and grat1's CF in doubles were all tried and recorded
  as negatives; the best form is still r3-bgrat's stored-z ratio, 246/1,881/2,357 of 2,908.
- Lead: bgrat may take Q(b, z) from Excel's own incomplete gamma (as GAMMA.DIST computes it) instead of TOMS grat1.
  The worksheet cannot show Q at b < 1 directly (T-2512: CHISQ.DIST.RT truncates its df, 1 - GAMMA.DIST loses the
  digits), so probe GAMMA.DIST(z, b, 1, TRUE) runs at small z where P is small and its own noise is visible, and
  compare its noise model with D's on the same (b, z) runs. Work in docs/research/t2464/ in a new `r6-*` directory,
  importing r3-bgrat's `bgld.py` (it now takes an `efn` and a `cfd` Q mode) and r5-bgrat-fdf1's `dreg.py` method.
- Rules: positive controls first; a real hold-out; record negatives (decision 0056); truncate formula literals to 15
  digits as Excel does.

## Acceptance criteria
- [ ] A RESULTS.md records a model of D with exact, within-4-ulp and within-15-digit counts on a held-out probe, or
      the negatives tried with their counts.
- [ ] New oracle probes are committed under eval/suites/probes-numeric/cases/ with their sidecars.
- [ ] statistics.md's "Incomplete beta: the integrated model" entry is updated with what is found.
- [ ] Engine follow-ups are filed for any solved piece.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes; F's arguments (T-3580); the other T-3468 items (T-3568 to T-3572).
