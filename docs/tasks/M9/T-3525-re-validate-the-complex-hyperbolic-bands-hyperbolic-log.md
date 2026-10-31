---
id: T-3525
title: "Re-validate the complex hyperbolic bands (HYPERBOLIC_LOG_CUT, INVERSE_EXP_LOW, INVERSE_EXP_HIGH) on a fresh hold-out sweep (decision 0188 R4)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3360, T-3361]
---

## Goal
The complex hyperbolic bands of engineering/complex/trig.rs (`HYPERBOLIC_LOG_CUT` 709, `INVERSE_EXP_LOW` 0.01, `INVERSE_EXP_HIGH` 20) are shown to generalize on a fresh hold-out sweep (IMSINH, IMCOSH and the functions built on them), or are replaced by the accurate form.

## Context
- Split from T-3365 (decision 0188's code audit, "tuned cut-points" and "others"; R4.1 hold-out split, R4.2 fresh
  sweep, R4.3 `FITTED` range). Functions that port Excel's algorithm generalize to fresh points; tuned selectors are
  the risk.
- `HYPERBOLIC_LOG_CUT` is `FITTED(T-2231, 84 cells, |x| 100..800 ...)`; the `1/u` band is
  `FITTED(T-2629, 306 cells, |x| 0.00012..100; cuts bracketed 0.01..0.0105 and 18.2..100)`: the upper cut was fitted
  inside a bracket of 18.2..100, so its placement there is unevidenced. engineering.md (T-2629) has the notes.
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
- The other selectors split from T-3365 (T-3502, T-3503, T-3523, T-3524, T-3525, one each); extrapolated fits (T-3364); the t, F and
  chi-squared inverses (T-3363).
