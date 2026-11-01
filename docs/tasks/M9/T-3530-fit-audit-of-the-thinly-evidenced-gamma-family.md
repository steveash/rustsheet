---
id: T-3530
title: "Fit audit of the thinly evidenced gamma-family FITTED constants (decision 0188 R4)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Scope as delivered: verdicts for the type-level rules, with their wrong-type cells handed to owning tasks; the six
bit-level constants' two-sided scoring moved to T-3633 (see Outcome). Original goal: every gamma-family `FITTED` constant with 6 or fewer evidence cells is either confirmed on fresh oracle evidence
(bracket narrowed, count raised), restricted or replaced where Excel disagrees inside its applied range, replaced by
accurate math where that does at least as well, or reclassified as one of Excel's own boundaries.

## Context
- Decision 0188 R4 (anti-overfitting guards): a fitted piece is not applied outside the range it was measured on.
- The constants (on `main` 744815d06): gamma.rs `HUGE_SHAPE_DENSITY_CUT`, `HUGE_SHAPE_CDF_CUT` (T-3271),
  `LOG_FORM_SHAPE_MAX`, `LOG_FORM_SHAPE_MIN` (T-2852); incomplete_gamma/huge_shape.rs `HUGE_SHAPE_MIN`,
  `BAND_UPPER_EDGE`, `BAND_LOWER_EDGE`, the closed-form density cut of `huge_shape_quantile` (T-3070);
  incomplete_gamma/mod.rs `RATIO_FORM_MIN` (T-2638), `CLOSED_FORM_QUANTILE_MAX`, `CLOSED_FORM_STEP_MIN` (T-3013;
  moved unchanged to closed_form.rs by T-3311); incomplete_gamma/density.rs `SMALL_RATIO_SHAPE_MIN` (T-3147).
- Probes: probes-numeric `fit-audit-*` (12 cases, 579 cells, 575 filled from excel-graph; `fit-audit-t3271-cdf-cut`
  one cell per workbook, A5, A8, A31 and A33 unfilled).
- Report: docs/research/fit-audit/README.md.

## Acceptance criteria
- [x] The type-level rules have a verdict (a)-(d) backed by the fit-audit probes, in the report: 6 of the 12
      constants (`HUGE_SHAPE_DENSITY_CUT`, `HUGE_SHAPE_CDF_CUT`, `HUGE_SHAPE_MIN`, `BAND_UPPER_EDGE`, `BAND_LOWER_EDGE`,
      the closed-form density cut). The other six have no verdict yet: T-3633.
- [x] No restriction or replacement lands here: the refuted rules' wrong-type cells are in their owners' Context
      (T-3316, T-3409, T-3396), which make the change. No (a) or (c) verdict, so no `FITTED` tag changes; the
      closed-form cut's tag is T-3633's.
- [x] No gated regression and no cell newly visibly off (decisions 0083, 0111): no engine code, and probes-numeric is
      ungated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Constants with more than 6 evidence cells.
- T-3265's fix of GAMMA.INV at a shape below 1 (this audit only supplies evidence for it).

## Handoff
Superseded by the Outcome. The earlier mid-audit notes (2026-10-31) are in git history.

## Outcome
Audited the 12 thinly evidenced gamma-family FITTED constants on 12 probes-numeric `fit-audit-*` cases (excel-graph;
579 cells, 575 filled). The cdf-cut case was filled one cell per workbook, because excel-graph refused the whole
case; A5, A8, A31 and A33 have no value. Re-scored on main 4e2f5f3df, after T-3350 changed the band's lower edge. The report
with per-case counts is docs/research/fit-audit/README.md. The branch has no engine code.
- Refuted (b), with wrong-type cells:
  - `HUGE_SHAPE_DENSITY_CUT`: 3 cells; at shapes 1E8 to 1E15 Excel's variable looks like x/shape (not at 1E70 and up).
  - `HUGE_SHAPE_CDF_CUT`: 21 of 38, both ways; Excel is not monotone in the shape.
  - `HUGE_SHAPE_MIN`: 8; Excel is `#NUM!` from shape 6E8.
  - The band's upper edge, plus the lower tail at 7E9 to 1.1E10: 16.

  Owners: T-3316 (raised to P1, `campaign: semantics`) for the two GAMMA.DIST cuts; T-3409 and T-3396 for
  GAMMA.INV. Each owner's Context lists the cells.
- No longer refuted: `BAND_LOWER_EDGE`. T-3350's cdf-ratio edge separates all 42 cells of fit-audit-t3070-band-lower;
  the old `offset - z < 39` rule had 12 wrong.
- Confirmed (d): the closed-form density cut of `huge_shape_quantile` is Excel's own boundary. 59 of 59 cells are
  within 15 digits.
- Provisional, moved to T-3633: `LOG_FORM_SHAPE_MAX`, `LOG_FORM_SHAPE_MIN`, `RATIO_FORM_MIN`,
  `CLOSED_FORM_QUANTILE_MAX`, `CLOSED_FORM_STEP_MIN` and `SMALL_RATIO_SHAPE_MIN` (two-sided scoring with a scratch
  build). The closed-form cut's tag update also moved there. T-3633 is filed.
- RATIO_FORM_MIN's region (46 of 62 beyond 15 digits on both sides of the cut) is evidence for T-3617 (filed with the
  T-3509 blind probe, branch claude/hopeful-meitner-5oxf0g-blind-probe; not on main yet).
- statistics.md: "GAMMA.DIST at a huge shape" (both cuts) and "GAMMA.INV and CHISQ.INV at a huge shape" (the band).
- probes-numeric is ungated, so the new cases change no baseline.

