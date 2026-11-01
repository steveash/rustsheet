---
id: T-3655
title: "MAKEARRAY: probe and match LAMBDA arity errors and the small and boundary sizes"
milestone: M7
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
MAKEARRAY gives Excel's error for a LAMBDA with the wrong number of parameters or a non-LAMBDA third argument, and Excel's result at the small and boundary sizes (0, 1, negative, fractional, text, boolean, errors).

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports MAKEARRAY errors for arity and boundary sizes. Done: T-2308 (non-whole sizes are
#VALUE!), T-2406 (array step is #CALC!), T-2408 (sizes past the grid, argument errors before size validation), T-1834
(lifted sizes); arrays.md "MAKEARRAY rows and columns must be whole numbers". Open: T-2345 (sizes within an ulp of
a whole number), T-2608 (where sizes past the grid are refused), T-2463 (LAMBDA element probes), T-2874 (empty text,
error and range argument kinds). Not probed: a LAMBDA with 0, 1, 3 parameters; a LAMBDA with an optional or omitted
parameter; a name bound to a LAMBDA; an eta-reduced function of the wrong arity (_xleta.ABS); a non-LAMBDA third
argument (a number, a reference, text); a missing third argument; and sizes 0, -1, 1, TRUE, "2", 1.0 written as
text. Compare how BYROW, BYCOL, MAP, SCAN and REDUCE treat a wrong-arity LAMBDA (`rg -i arity docs/excel-semantics crates/rustsheet-engine/src/functions/logical`), and reuse any existing arity check rather than adding one.
Code: `fn_makearray` in functions/logical/bycol_byrow.rs.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the arities and sizes above.
- [x] The engine matches every probed cell, sharing one arity check with the other LAMBDA helpers; arrays.md states it.
- [x] Cells in T-2345, T-2608 or T-2874's scope are left to them.
- [x] Every quirk found is written up in docs/excel-semantics/arrays.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- Sizes past the grid (T-2608).

## Outcome
Probes (excel-graph, 2026-11-01): probes/t3655-makearray-arity-sizes (62 formulas: arities, non-LAMBDA third
arguments, sizes, size against bad function, the other helpers) and probes/t3655-lambda-helper-function-argument (27:
error and non-LAMBDA functions in MAP/SCAN/REDUCE/BYROW/BYCOL, size-vs-function order, functions passed by name of the
wrong count, optional parameters). Finding: every LAMBDA helper is one whole-call `#VALUE!` for a function it cannot
call with its argument count, an error argument included (`MAKEARRAY(2,2,NA())` is `#VALUE!`); optional parameters
fill the count; MAKEARRAY checks sizes first. The engine gave an array of per-element `#VALUE!` for a wrong count and
passed an error argument on. Now one check, `Model::helper_caller` (lambda.rs), feeds MAKEARRAY, MAP, BYROW, BYCOL,
SCAN and REDUCE (BYROW's own `by_slices_lambda` is gone). Sizes (0, negative, fractions below 1, TRUE, FALSE, numeric
text, references) already matched. arrays.md: "LAMBDA helpers refuse a function they cannot call", "MAKEARRAY sizes:
zero, negative, booleans, text and references". Eval: the two new probe cases pass (421 cells), no other change.
