---
id: T-3633
title: "Score the six provisional gamma FITTED cuts on both sides of their brackets (T-3530 fit audit, part 2)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3530]
---

## Goal
Each of the six gamma-family FITTED cuts T-3530 left provisional has a decision 0188 R4 verdict, (a) to (d), from
counting bit-exact cells on each side of its bracket. The closed-form density cut's tag is narrowed to Excel's
boundary.

## Context
- T-3530 (docs/research/fit-audit/README.md) probed these with excel-graph but scored them only at the current value:
  - gamma.rs `LOG_FORM_SHAPE_MAX` 3.0 and `LOG_FORM_SHAPE_MIN` 0.5 (T-2852): fit-audit-t2852-shape-max (52 cells,
    4 off: A25, A29, A34, A46) and fit-audit-t2852-shape-min (48 cells, 0 off);
  - incomplete_gamma `RATIO_FORM_MIN` 0.005 (T-2638): fit-audit-t2638-ratio-min, 46 of 62 off on both sides of the
    cut. The region itself is T-3617's.
  - closed_form.rs `CLOSED_FORM_QUANTILE_MAX` 1E-21 and `CLOSED_FORM_STEP_MIN` 3.5E-14 (T-3013):
    fit-audit-t3013-quantile-max (18 of 54 off) and fit-audit-t3013-step-min (8 of 20 off, straddling the cut). Both
    are evidence for T-3265, which owns the step model.
  - density.rs `SMALL_RATIO_SHAPE_MIN` 3.0 (T-3147): fit-audit-t3147-ratio-shape (3 of 56 off on main 4e2f5f3df:
    A12, A15, A28).
- Method: a scratch build in which each of the six constants is read from an environment variable, falling back to
  its current value. Never commit it. Build it in a private worktree and copy the binary out. Run each fit-audit case
  with the constant at its current value and at the other end of its bracket (the brackets are in the report's
  first table), and count bit-exact cells on each side of the cut (`rustsheet-eval show --all`). Then verdict (a)
  (confirmed, narrow the bracket), (b) (refuted), (c) (math does as well) or (d) (Excel's own boundary).
- The closed-form density cut of `huge_shape_quantile` (huge_shape.rs, T-3070) was confirmed as Excel's own
  boundary (verdict d) on 59 fresh cells (fit-audit-t3070-closed-form-cut). Its FITTED tag is to be narrowed and its
  count raised to 61 through `check_fitted.py --update`.

## Acceptance criteria
- [ ] Each of the six cuts has a verdict with the per-side counts in docs/research/fit-audit/README.md.
- [ ] Verdicts (a) and (c) land with updated `FITTED` tags (quality/fitted.json via `check_fitted.py --update`); a
      replacement (b) is its own task.
- [ ] The closed-form density cut's tag reads 61 cells and Excel's boundary.
- [ ] No gated regression and no cell newly visibly off (decisions 0083, 0111).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The wrong-type rules (T-3316, T-3409, T-3396) and T-3265's step model.
