---
id: T-3313
title: "Move the closed-form gamma quantiles into an incomplete_gamma::closed_form submodule (mod.rs is near its 1,000-line limit)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/mod.rs` has room under the 1,000-line file
limit again. The closed-form quantile code moves, unchanged, into a new `incomplete_gamma/closed_form.rs` submodule.

## Context
T-2561 added `leading_factor_root`, `TINY_QUANTILE_CHECK` and `excel_tiny_quantile` next to T-3013's
`excel_small_shape_quantile`, `CLOSED_FORM_QUANTILE_MAX` and `CLOSED_FORM_STEP_MIN`. That brought mod.rs to 997
lines (T-2561 architecture review, round 1). T-3070 set the pattern: it moved the huge-shape path into
`incomplete_gamma/huge_shape.rs` with a `pub(crate) use` re-export.

## Acceptance criteria
- [ ] The six items above (and their doc comments) live in `incomplete_gamma/closed_form.rs`. Callers (`gamma.rs`,
      `gamma_quantile`, tests) use them through `mod.rs`, as `huge_shape` is used. The code is not changed.
- [ ] The full eval comparison of main against HEAD shows no cell changed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what the functions compute.

## Outcome
Done as the first code commit of T-3311's PR (3a3a4102f, "move the closed-form gamma quantiles into
incomplete_gamma::closed_form"): the six items moved unchanged into `incomplete_gamma/closed_form.rs`, re-exported
through `mod.rs` as `huge_shape` is; quality/fitted.json moves three FITTED tags from mod.rs to closed_form.rs (total
unchanged). Its eval delta is not separate from T-3311's: every cell the PR changes is in T-3311's comparison and comes
from `excel_newton_quantile`, which T-3311 added after the move.
