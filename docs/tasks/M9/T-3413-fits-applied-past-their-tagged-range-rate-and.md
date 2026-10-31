---
id: T-3413
title: "Fits applied past their tagged range: RATE and IRR secant constants, F density SMALL_SHAPE_OFFSET below LARGE_DF_MIN (R4.3)"
milestone: M9
lane: quality
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Each fit that T-3395 showed applied past the range its `FITTED` tag names either gets measured over the domain it is
used on, under decision 0188 R4's guards, or is bounded to its range with an accurate fallback outside it (R4.3).

## Context
T-3395 (decision 0195, reviewer-excel-fidelity MINOR) tagged these fits. Their tags name ranges narrower than the
inputs they serve:
- crates/rustsheet-engine/src/functions/financial/financial_util.rs `compute_rate`: `OFFSET` and the residual use of
  `TOLERANCE` (fit lab `rate`, 782 cells: nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10, rate -0.75..2) apply to
  every RATE call.
- `irr_secant`: the 1E-3 start offset and 1E-7 residual stop (fit lab `irr`, 263 cells: 2..41 flows, |c| 1E-6..1E+9,
  guess -0.99..100, roots -0.99..99) apply to every IRR call.
- crates/rustsheet-engine/src/functions/statistical/f_density.rs `SMALL_SHAPE_OFFSET`, `STIRLING_ERROR_HALF` and
  `STIRLING_ERROR_ONE` (4,946 cells, df 1..10 and d1 1 up to d2 30) apply to every shape with min(d) below
  `LARGE_DF_MIN` (16), including 10 < min(d) < 16 and max(d) > 10.
For a solver parameter, "accurate math" outside the range is a design question (the secant's trajectory, not a
formula, makes the value), so a fresh sweep over the wider domain may be the right answer rather than a fallback.
T-3364 covers other extrapolated fits (GAMMA.INV, the Stirling error past df 56, the Student-t df bands), not these.

## Acceptance criteria
- [ ] For each fit above: a fresh log-uniform sweep over the domain it serves (R4.2) with the Gate A envelope result,
      and either the tag's range widened to that sweep or the fit bounded to its range with the fallback scored.
- [ ] Tags and decision 0195's consequences updated; no gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The Student-t cuts (T-3410) and T-3364's fits.
