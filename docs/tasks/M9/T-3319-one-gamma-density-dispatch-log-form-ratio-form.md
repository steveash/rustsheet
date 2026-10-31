---
id: T-3319
title: "One gamma density dispatch: log form, ratio form, then direct, in incomplete_gamma/density.rs"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3147, T-2852]
---

## Goal
One gamma density entry point: `incomplete_gamma::gamma_density` (incomplete_gamma/density.rs, T-3147) tries Excel's
log form (T-2753, T-2852), then the ratio form (T-3147), then the direct prefactor, so GAMMA.DIST and CHISQ.DIST call one
function and every shape split is one named constant. No cell changes value (a pure refactor).

## Context
After T-3147 and T-2852 the density is chosen in two places: gamma.rs `gamma_dist_of` and chisq.rs both write
`log_form_density(..).or_else(|| gamma_density(..)).unwrap_or_else(|| dist.pdf(x))`, the log form living in gamma.rs and
the ratio form in incomplete_gamma/density.rs. The shape bounds also overlap: gamma.rs `LOG_FORM_SHAPE_MAX` (where the log
form stops; `LOG_FORM_ABOVE_MODE_SHAPE_MAX` = 2.5 above the mode) and density.rs `SMALL_RATIO_SHAPE_MIN` (where the ratio form above the mode starts) are separate fits of
neighbouring brackets and should become one constant, or two that name each other, with their brackets restated
(statistics.md "GAMMA.DIST's density below shape 3 is a log-space form" and the T-3147 paragraph of "Incomplete gamma:
accurate kernel"). Review finding on T-2852 (architecture reviewer, round 1).

## Acceptance criteria
- [ ] GAMMA.DIST, GAMMADIST, CHISQ.DIST call one density function; the `or_else` chain appears once.
- [ ] The shape split between the log form and the ratio form is one named constant (or two that reference each other).
- [ ] The main-vs-HEAD per-cell comparison shows no cell changing value.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change of form or bound (T-3303, T-3219).
