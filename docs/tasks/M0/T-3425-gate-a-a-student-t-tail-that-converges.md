---
id: T-3425
title: "Gate A: a Student t tail that converges at df above 5E6 in the generator"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3357]
---

## Goal
`rustsheet-accuracy generate T.INV` and `T.DIST` keep their large-df points: the Student t lower tail in
`tools/src/rustsheet_tools/accuracy/defs/student_t.py` computes at 50 digits for df up to 1E10 at every x the
sampler draws, so no point is dropped for non-convergence.

## Context
- T-3357's generator drops a point when mpmath raises `NoConvergence` (SPEC-0029's drop path). `mpmath.betainc`
  goes through a hypergeometric series that stalls for df above about 5E6 when y = df/(df + x²) is near 1: a trial
  T.INV `s1` batch dropped 11 of 120 points, all at df from 5.4E6 to 1E10 (extreme and log-uniform strata, and the
  df = 1E10 edge), e.g. `T.INV(0.00103896513665999,5395690)` and `T.INV(5.13945494250381E-214,3344838607)`.
- Those are the points decision 0188 cares about (large-df tails); T-3358's T.INV/T.DIST tables need them.
- A direct 2F1 series costs about 1/z = (df + x²)/x² terms and the complement about x²/2 terms with a cancellation
  of the tail's digits; a choice between them with adaptive extra precision, or a continued fraction, or an
  asymptotic expansion in 1/df with a cited error bound are candidates. Keep the dps 60/80 convergence check.

## Acceptance criteria
- [ ] A T.INV and a T.DIST batch of 120 points drop no point for non-convergence (or name each remaining drop and why).
- [ ] Unit tests at df 1E7 and 1E10 against an independent reference (the normal limit with the Cornish-Fisher or
      Edgeworth correction to its cited order, or quadrature of the density) agree to 1e-40.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other distributions' kernels (T-3358 slices).
