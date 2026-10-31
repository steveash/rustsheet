---
id: T-3365
title: "Re-validate the tuned form selectors on a fresh hold-out sweep and simplify the ones that do not generalize"
milestone: M9
lane: quality
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3360, T-3361]
---

## Goal
Each tuned form selector of decision 0188's code audit is scored on a fresh hold-out sweep (R4.1, R4.2); a selector that
does not generalize is simplified to the accurate form, and one that does keeps its `FITTED` tag with the measured range.

## Context
- Owner's 2026-10-31 assessment (decision 0188, code audit, tuned cut-points and others):
  - financial/annuity.rs `DISCOUNTED_INTEREST_FROM`;
  - financial/cumulative.rs `GEOMETRIC_MIN_LEAD` and `LONG_GEOMETRIC_MIN_LEAD`;
  - engineering/complex/trig.rs bands (`HYPERBOLIC_LOG_CUT`, `INVERSE_EXP_LOW`, `INVERSE_EXP_HIGH`);
  - statistical/geomean.rs the 15-digit snap of the root;
  - statistical/gammaln.rs the K2 dual coefficients (`K2_CAND9_BELOW`, gamma_kernels.rs `K2_CAND9_*`).
- Functions that port Excel's algorithm generalize to fresh points (PMT, NPER, GAMMALN, Bessel); tuned selectors are the
  risk. Lane quality because the sites span four function lanes; each change stays in its own module.

## Acceptance criteria
- [ ] For each selector, the Outcome reports the fit lab's train and test counts (T-3360) and a fresh log-uniform sweep's
      result (R4.2), with the selector and with the accurate form.
- [ ] A selector whose test score is not within the stated margin of its train score, or that does not beat the accurate
      form on test, is replaced by the accurate form; the rest keep a `FITTED` tag with their range (T-3361).
- [ ] The Gate A envelope holds on the fresh sweeps (decision 0188 R2).
- [ ] No gated cell regresses beyond decision 0083's allowance.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The extrapolated fits (T-3364) and the t, F and chi-squared inverses (T-3363).
