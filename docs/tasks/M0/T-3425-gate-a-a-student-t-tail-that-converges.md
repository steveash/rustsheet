---
id: T-3425
title: "Gate A: a Student t tail that converges at df above 5E6 in the generator"
milestone: M0
lane: harness
status: done
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
- [x] A T.INV and a T.DIST batch of 120 points drop no point for non-convergence (or name each remaining drop and why).
- [x] Unit tests at df 1E7 and 1E10 against an independent reference (the normal limit with the Cornish-Fisher or
      Edgeworth correction to its cited order, or quadrature of the density) agree to 1e-40.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other distributions' kernels (T-3358 slices).

## Outcome
Done (2026-11-01).
- **Cause:** T.INV's root search (`lower_quantile`) evaluates the tail across x = -e^-60 to -e^710. For a df above
  about 5E6, mpmath's `betainc` (a hypergeometric series) stalls from |x| ≈ 90 up to about √df. Those tails are below
  e^-4000, beyond any 15-digit p, but the search still evaluates them, so the point was dropped.
- **Fix** (`tools/src/rustsheet_tools/accuracy/defs/student_t.py`):
  - From |x| = 2: I_y(df/2, 1/2) by its continued fraction (DLMF 8.17.22, modified Lentz) with a log-space prefactor
    and 20 guard digits. It converges for x² ≥ 3 at every df, in at most about 600 terms (df 1E10 at |x| = 2) and
    fewer as |x| grows.
  - The fraction is one shared `betacf` (`fitlab/kernels/beta.py`, generic over Decimal and mpmath). The fitlab
    70-digit reference now calls it.
  - Below |x| = 2: the complement 1/2 - I_z(1/2, df/2)/2 with z = x²/(df + x²). The old form passed y = df/(df + x²),
    which rounds to 1 at df 1E10 and x = -e^-60, so it lost x² there (found by the new bracket test).
  - The density's guard digits grow with log10(df): at df 8.3E+129 the log-gamma difference cancelled past 60 digits,
    and T.DIST dropped 3 points for it.
- **Tests** (`tools/tests/test_accuracy_student_t_large_df.py`):
  - At df 1E7 and 1E10 and x = -0.5, -3 and -30: the tail agrees with mpmath's `betainc` at 200 digits to 1e-40.
  - At df 1E7 and x = -100, in the stalled band: it agrees with the complementary series at 2,260 digits to 1e-40.
  - Across the root search's bracket at df 5395690 and 1E10: the tail is finite and agrees at 60 and 80 digits.
  - Two of the task's dropped T.INV points now have quantiles.
  - The density agrees at 60 and 80 digits at df 8.3E+129.
  - Density quadrature was not used as the reference: mpmath's `quad` misses even the normal tail at x = -30 by 3E-9
    to 8E-7.
- **Batches:** `e-T-3425` for T.INV and T.DIST, 120 points each with no point dropped, filled from excel-graph. The
  existing `s1` tables recompute with 0 differences.
- **Known failures:** the new batches show four existing engine gaps.
  - T.INV A74 at df 1E10: the engine is 2.5E-6 off; Excel matches the math. Owner T-3363.
  - T.DIST A90, A103 and A116: density points at a huge x, where Excel is `#NUM!` and the engine returns 0. Owner
    T-3554.
- **Accuracy:** 0 failing, 0 table defects.
- **probes-numeric math sidecars:**
  - 130 more Student t cells get a math value: "not converged or too slow" falls from 185 to 55. No existing value
    changed.
  - The probes-numeric R1(c) count falls from 39527 to 39510; Phase 1b evidence rises from 32150 to 32167.
