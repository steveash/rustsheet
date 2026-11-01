---
id: T-3597
title: "BETA.DIST and F.DIST at basym points: Excel's arithmetic inside basym (the t3443 probes' visibly-off cdf cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
BETA.DIST, F.DIST and F.DIST.RT at TOMS 708 bratio's basym points (both shapes above 100, |λ| <= 0.03 min(a, b)) are
within 15 digits of Excel on the t3443 probes, from a model of Excel's arithmetic inside basym (or the cells are
recorded as UNVERIFIED with the hypotheses ruled out).

## Context
- T-3443 put these callers on the basym port (`bratio/basym.rs`, in doubles). Visibly off after it, in-sample /
  hold-out: BETA.DIST + BETADIST 33 / 20 of 50, F.DIST + F.DIST.RT + FDIST 17 / 23 of 40
  (probes-numeric/t3443-basym-a, t3443-basym-hold; statistics.md "BETA.DIST and F.DIST at basym points").
- Excel is basym-like but not ours to the last digit: against 50-digit math Excel is closer than ours on 26 of the
  in-sample cdf cells and further on 44, up to 3.3E-13 relative off (A26 `BETA.DIST(0.995975869785456,40036,161,TRUE)`)
  where ours is 1.3E-13, both inside basym's 100·eps stop. Candidates: the stop test and term count, x87 storage of
  the coefficient recurrences, `erfc1`, `rlog1` and `bcorr` (T-3447 owns the same question for BINOM.DIST).
- The BETA.INV and F.INV round trips' visibly-off cells follow the cdf's last digits; BETA.INV's nearest-end rule at
  these points also misses 5 in-sample cells (A136, A151, A155, A170, A173), which may move with the cdf.
- Also owned here: F.INV's exact-count losses from T-3443's kernel switch (in-sample 5 to 4, hold-out 5 to 3, each
  still within 15 digits), and BETA.INV's crossing rule at basym points. T-3443 takes the nearest bracket end there
  (+10 / +11 / +5 exact on the three cases against the even end). UNVERIFIED hypothesis, worth testing on all three
  cases and off basym: Excel searches the tail bratio computes directly rather than its complement (0.5 - w) + 0.5,
  which names H29's mechanism (the complement one ulp above fl(1 - p)).
- Method: docs/research/excel-kernel-playbook.md; a fit owes decision 0188 R4's guards.

## Acceptance criteria
- [ ] A model of Excel's basym arithmetic scored on t3443-basym-a (in-sample) and then once on t3443-basym-hold, with
      exact / within-15-digit counts, or the negatives tried with their counts in statistics.md.
- [ ] Decision 0188 R4 guards for any fitted piece (hold-out counts, a fresh sweep, Gate A envelope, `FITTED` tag and
      range, reproducibility floor, no keep-matched branch).
- [ ] No gated cell regresses; `make check` green; reviewers approve.

## Out of scope
- BINOM.DIST's and NEGBINOM.DIST's basym last digits (T-3447), though a shared model would close both.

## Visible-parity stage
Decisions 0083 and 0114 (`visible`): done when every visibly-off cdf cell of the two t3443 cases is within half a
unit of the 15th significant digit of Excel, or is an UNVERIFIED entry with its ruled-out hypotheses.
