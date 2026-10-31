---
id: T-3111
title: "Snap the near-zero root of a LAMBDA body at every call made by MAP, MAKEARRAY, BYROW, BYCOL, REDUCE and SCAN, and of an in-place call passed as a LAMBDA argument"
milestone: M7
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3105]
campaign: semantics
---

## Goal
Every call that MAP, MAKEARRAY, BYROW, BYCOL, REDUCE and SCAN make to their LAMBDA snaps a near-zero `+`/`-` root of
the body, as an in-place call does, so each element and each accumulator step holds Excel's 0 or Excel's residue. The
same goes for an in-place LAMBDA call passed as an argument to another LAMBDA call.

## Context
- Decision 0188 (owner's 2026-10-31 rulings, R1(c)): this task owns probe-row or text cells of its milestone's exit,
  which Phase 1 still counts in full, so it is in the semantics stage at P1 (was P3).
- Evidence: probes/t3095-snap-lambda-helpers (T-3095, excel-graph 2026-10-29); docs/excel-semantics/numbers.md
  "Near-zero snapping", paragraph "Pickers, LET, LAMBDA and unary signs". Excel snaps the body's root at every
  helper call, and REDUCE and SCAN snap at every step, so a cancelled accumulator is 0 before the next step:
  `REDUCE(0,{0.1,0.2,-0.3,1E-16},LAMBDA(a,x,a+x))` is 1E-16, not 1.5551115123125782e-16.
- Failing cells (known failures in eval/baseline/probes.txt), Calc!B:
  - MAP (2-5, 22-25, 62-65) and MAKEARRAY (7-9, 27-29, 67-69);
  - BYROW and BYCOL with a `SUM(r)-c` or an `INDEX(r,1)+INDEX(r,2)-c` body (13-20, 33-40, 73-80);
  - REDUCE and SCAN (81, 82, 85, 88, 89, 92, 93, 97, 100, 104, 105, 108, 111, 112, 115, 116, 135, 136);
  - an in-place call as a LAMBDA argument (94, 117: `LAMBDA(a,x,a+x)(LAMBDA(a,x,a+x)(..),..)` nested four deep).
  The k = 8 rows (41-60, 119-133) keep their residue and already pass.
- Code: `Model::call_lambda` (functions/logical/lambda.rs) sets `LambdaCaller::root_snap` for an in-place call only.
  `Model::lambda_caller` gives the helpers `root_snap: false` (bycol_byrow.rs, map_reduce.rs, scan.rs, MAKEARRAY).
  The snap itself is `Model::evaluate_formula_root` / `handle_outermost_sum` (arithmetic.rs). Reuse it; do not add a
  second snap. T-3105 changes the same flag for defined-name calls, so land after it.

## Acceptance criteria
- [ ] Every cell of probes/t3095-snap-lambda-helpers passes (`make eval-baseline` drops its 65 known failures), no
      regressions.
- [ ] numbers.md's paragraph says the engine matches the helper cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GROUPBY/PIVOTBY aggregation LAMBDAs and eta-reduced helpers (`MAP(r,SUM)`): probe first (see the T-3095 Outcome).
