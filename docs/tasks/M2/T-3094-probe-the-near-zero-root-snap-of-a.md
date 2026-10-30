---
id: T-3094
title: "Probe the near-zero root snap of a LAMBDA reached through a defined name, a LET name, a grouped body or a nested call"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for whether a LAMBDA body whose root is `+`/`-` snaps its near-zero result when the LAMBDA is not
written and called in place: bound to a defined name, bound by LET, written as a group, or called from another
LAMBDA. Each form is recorded per cell, so the engine snaps exactly where Excel does.

## Context
- docs/excel-semantics/numbers.md "Near-zero snapping", paragraph "Pickers, LET, LAMBDA and unary signs" (T-3049,
  probes/t3049-snap-under-pickers, excel-graph 2026-10-28): the snap follows the tree root of a cell formula, of a
  defined-name body and of a LAMBDA body called in place (`=LAMBDA(x,x+0.2-0.3)(0.1)` is 0, and k = 8 keeps
  1.7763568394002505e-15), while the same body under LET (`LET(x,0.1,x+0.2-0.3)`) is not snapped. The paragraph ends
  "Unprobed: a LAMBDA bound to a defined name, LAMBDA helpers (MAP, REDUCE, BYROW) and a LAMBDA body written as a
  group". This task takes the first and the last; the helpers are T-3095 (M7).
- T-3083 (engine, in progress) snaps the in-place call in `Model::call_lambda` (functions/logical/lambda.rs) through
  `Model::evaluate_formula_root` / `handle_outermost_sum` (arithmetic.rs). Its out-of-scope note says to probe the
  other call forms first; this is that probe. The two readings these forms tell apart: "every LAMBDA call snaps its
  body's root" (the call is the root) against "only a call that is itself the cell's root does" (`=f(0.1)+0`
  would then keep the residue, as `=LET(x,..,x)+0` does).
- Why opus: the answer decides where the snap is applied for every LAMBDA caller (named, LET-bound, recursive),
  and the rows need the k = 7 / k = 8 boundary (`1+k*2^-52-1`; T-3047 found the `-` side snaps only up to k = 3) so that a
  "snapped" reading is not confused with a residue that happens to be 0. Groups also interact with the name-body rule
  (T-1164: a name body written as a group is not snapped).

## Acceptance criteria
- [x] A new excel-graph probe case `probes/t3094-snap-lambda-call-forms` (milestone M2, decision 0043; not M1) reads
      back, for the bodies `x+0.2-0.3` at 0.1, `x+7*2^-52-1` and `x+8*2^-52-1` at 1, and `-x-0.2+0.3` at 0.1, each of:
      a workbook-scoped defined name `f` = `=LAMBDA(x,<body>)` called as `=f(..)`, `=f(..)+0`, `=(f(..))` and
      `=SUM(f(..))`; `=LET(g,LAMBDA(x,<body>),g(..))`; a grouped body `=LAMBDA(x,(<body>))(..)`; a LAMBDA whose body
      calls another (`=LAMBDA(y,LAMBDA(x,<body>)(y))(..)`, and a named `h` = `=LAMBDA(y,f(y))`); a body under a picker
      (`=LAMBDA(x,IF(TRUE,<body>))(..)`); and a body that is a LET (`=LAMBDA(x,LET(z,x,<body with z>))(..)`). The
      in-place call `=LAMBDA(x,<body>)(..)` is the control in every row.
- [x] numbers.md "Pickers, LET, LAMBDA and unary signs" records, per form, whether the body's root is snapped, citing
      the cells, and its "Unprobed" sentence keeps only the helper forms (T-3095).
- [x] Every cell the engine does not match is a known failure in eval/baseline/probes.txt, and one follow-up task
      (milestone M2, lane engine, tier opus) names those cells and depends on T-3083; `make exit-owners MILESTONE=M2`
      reports 0.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes: the in-place call is T-3083's, the other forms the follow-up's.
- LAMBDA helpers (MAP, BYROW, BYCOL, REDUCE, SCAN, MAKEARRAY): T-3095.
- Sheet-scoped names and recursion deeper than one level.

## Outcome
Probe probes/t3094-snap-lambda-call-forms (excel-graph, 44 scored cells): every call of a LAMBDA snaps its body's
near-zero `+`/`-` root, whether it is called in place, through a workbook-scoped name (bare, `+0`, grouped, under SUM),
LET-bound, nested or through a named caller. What decides is the body's own shape: a grouped body, an `IF` body or a
LET body keeps the residue, and k = 8 keeps it in every form. This is recorded in numbers.md "Pickers, LET, LAMBDA and
unary signs". Eval delta: one new case, 29 of 44 cells passing, and 15 new known failures (the defined-name calls
C:F and J in rows 2, 3 and 5). Follow-up T-3105 (engine, opus, depends on T-3083) owns them. No regressions.
