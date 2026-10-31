---
id: T-3361
title: "FITTED(T-XXXX, n_cells, range) doc tag: tag the known fitted constants and ratchet their count in make hygiene (decision 0188 R4.3)"
milestone: M0
lane: quality
status: todo
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
  - statistical/t_dist.rs `T_TAIL_P`, `T_TAIL_X`, `T_STATRS_MIN_P`;
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
