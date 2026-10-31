---
id: T-3361
title: "FITTED(T-XXXX, n_cells, range) doc tag: tag the known fitted constants and ratchet their count in make hygiene (decision 0188 R4.3)"
milestone: M0
lane: quality
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Every empirically fitted constant, cut-point or table in the engine carries a `FITTED(T-XXXX, n_cells, range)` doc tag
(the task that fitted it, the cells it was measured on, and the input range they cover), a script counts the tags and
`make hygiene` ratchets the count, so new fits are visible and each one's range is written down.

## Context
- Decision 0188 R4.3 (owner's 2026-10-31 assessment): about 15 empirically fitted continuous constants or tables, some
  applied outside the range they were measured on.
- Known sites for the first tagging pass (crates/rustsheet-engine/src/functions/):
  - financial/annuity.rs `DISCOUNTED_INTEREST_FROM`;
  - financial/cumulative.rs `GEOMETRIC_MIN_LEAD` and `LONG_GEOMETRIC_MIN_LEAD`;
  - financial/financial_util.rs the IRR fallback (`irr_fallback`, T-2727);
  - statistical/student_t.rs `LOG_GAMMA_BETA_MIN` (6.5) and `LOG_GAMMA_BETA_MAX`, and the df bands;
  - statistical/t_dist.rs `T_QUANTILE_SERIES_DF`, `T_TAIL_P`, `T_TAIL_X`, `T_STATRS_MIN_P`;
  - statistical/inverse_beta.rs the probe-tuned thresholds of the statrs port;
  - statistical/incomplete_gamma/huge_shape.rs `WILSON_HILFERTY_SPREAD` and its 1/a, z²/a, z³/a^(3/2) terms (fitted
    at 1E13, applied from 2E10);
  - statistical/f_density.rs `STIRLING_ERROR` (applied past df 56), `STIRLING_ERROR_SLOPE`, `LARGE_SHAPE_OFFSET`,
    `SMALL_SHAPE_OFFSET`, `STIRLING_ERROR_HALF`, `STIRLING_ERROR_ONE`;
  - statistical/gammaln.rs and gamma_kernels.rs the K2 dual coefficients (`K2_CAND9_*`, `K2_CAND9_BELOW`);
  - statistical/geomean.rs the 15-digit snap of the root;
  - engineering/complex/trig.rs the bands (`HYPERBOLIC_LOG_CUT`, `INVERSE_EXP_LOW`, `INVERSE_EXP_HIGH`).
  Search for more (`rg -n 'fitted|fit lab|measured on' crates/rustsheet-engine/src`); a constant derived from Excel's
  documented algorithm or from math is not a fit and gets no tag.
- The ratchet follows the existing ones (scripts/check_lint_debt.py, quality/*.json written only by `--update`).
  scripts/ and quality/ are protected paths: this task's PR cites decision 0188.

## Acceptance criteria
- [ ] Every site above (and any found by the search) has a `FITTED(T-XXXX, n_cells, range)` doc comment with real values
      from its task's evidence; a site whose range is unknown says `range unknown` and gets a follow-up task.
- [ ] A script in scripts/ counts the tags (and rejects a malformed one); `make hygiene` runs it against a committed
      count that may only fall, or rise in a PR that cites a fit task and its hold-out result (decision 0188 R4).
- [ ] Tests for the script; `make check` green; reviewers approve.

## Out of scope
- Changing any fitted value or adding the out-of-range fallbacks (T-3364, T-3365).

## Outcome
- 80 `/// FITTED(T-NNNN, <n> cells, <range>)` tags on 25 files under crates/rustsheet-engine/src/functions/: every
  listed site the evidence shows was fitted, plus the fits the search found (financial form cuts and solver stops,
  the Bessel asymptotic tables, the hyperbolic switches, the GAMMALN kernels, the gamma, Poisson, chisq and exponential
  cuts). Counts and ranges come from each task's Outcome note, its probe cases or the cells the item's doc cites.
- Not tagged, because the value is math, a published algorithm, an Excel limit or a directive-5 cap, not chosen on
  Excel cells: t_dist.rs `T_QUANTILE_SERIES_DF` (a statrs hang guard, T-1518) and `T_TAIL_X` (the asymptotic's
  truncation error); inverse_beta.rs, audited whole (its step caps are directive-5 limits, the AS 64/109 terms are
  the published algorithm and `GAMMA_LIMIT_SHAPE_RATIO` is an mpmath error bound; the "probe-tuned thresholds" are the
  t_dist.rs `T_TAIL_P` and `T_STATRS_MIN_P`, which are tagged); the TOMS 708 / AS 241 constants.
- Five tags say `range unknown` (irr_fallback, compute_xirr, excel_yield_newton, SERIES_MIN_SHAPE_SUM,
  F_QUANTILE_REFINE_P_MIN): follow-up T-3385, which also settles hyperbolic.rs `EXPM1_FORM_LIMIT`.
- Fits applied past their measured range, now visible in the tags: f_density.rs `STIRLING_ERROR` (measured on
  BETA.DIST, applied to F.DIST past v 28), yield_solver.rs `yield_start` (price 90.5..99.5), t_dist.rs `T_TAIL_P` and
  `T_STATRS_MIN_P` (probe p up to 1E-10, cuts at 1E-5 and 1E-8) — T-3364's subject.
- Ratchet: scripts/check_fitted.py (+ tests) in `make hygiene` against quality/fitted.json (80). Eval delta: none.
