---
id: T-3503
title: "Re-validate GAMMALN's K2 dual coefficients (K2_CAND9_BELOW) on a fresh hold-out sweep (decision 0188 R4)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3360, T-3361]
---

## Goal
GAMMALN's K2 dual coefficients (statistical/gammaln.rs `K2_CAND9_BELOW` 0.01 and gamma_kernels.rs `K2_CAND9_N`, `K2_CAND9_D`) are shown to generalize on a fresh hold-out sweep near x = 2, or are replaced by the single K2 kernel.

## Context
- Split from T-3365 (decision 0188's code audit, "tuned cut-points" and "others"; R4.1 hold-out split, R4.2 fresh
  sweep, R4.3 `FITTED` range). Functions that port Excel's algorithm generalize to fresh points; tuned selectors are
  the risk.
- Below |x-2| < 0.01 a second, fitted coefficient set replaces K2; statistics.md has the measured optimum. The
  dual set is a fit of coefficients, so a fresh sweep near 2 (log-uniform in |x-2| from 1E-15 to 0.05, both sides)
  is the generalization test. GAMMALN.PRECISE and GAMMA paths that share the kernel are in scope.
- Method: build the engine twice (with the selector, and with the accurate form only) and score both on the old
  cells through the fit lab (`docs/architecture/fit-lab.md`, hold-out split, default margin 5 points), then on the
  fresh sweep. Accuracy-suite tables are refused as sweeps (`--holdout-only` measures a frozen fit only).

## Acceptance criteria
- [ ] A fresh log-uniform sweep over the selector's full domain (R4.2), new cells in probes-numeric that no earlier fit
      saw, is oracle-filled and committed.
- [ ] The Outcome reports, with the selector and with the accurate form (two engine builds, or fit-lab candidates that
      reproduce each bit for bit on the old cells), the fit lab's train and test counts (T-3360's hold-out split and
      margin) on the old cells and the counts on the fresh sweep.
- [ ] If the selector's test exact rate is not within the stated margin of its train rate, or it does not beat the
      accurate form on test and on the fresh sweep, it is replaced by the accurate form (one `FITTED` tag fewer in
      quality/fitted.json via `scripts/check_fitted.py --update`); otherwise it keeps its `FITTED` tag, with the range
      restated to what was measured, and is not applied outside that range (R4.3).
- [ ] The Gate A envelope (R2: within 1e-9 relative of 50-digit mpmath or within 15 significant digits of Excel) holds
      on the fresh sweep, with mpmath values recorded in the Outcome or the accuracy tables.
- [ ] No gated cell regresses beyond decision 0083's allowance; a replaced selector's lost bit-exact cells are listed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other selectors split from T-3365 (T-3499..T-3503, one each); extrapolated fits (T-3364); the t, F and
  chi-squared inverses (T-3363).
