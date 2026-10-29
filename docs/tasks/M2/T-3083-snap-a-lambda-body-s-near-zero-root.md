---
id: T-3083
title: "Snap a LAMBDA body's near-zero root + or - at its own root"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A LAMBDA body whose root is `+`/`-` snaps a near-zero result to 0 when the LAMBDA is called, as Excel does,
so `=LAMBDA(x,x+0.2-0.3)(0.1)` is 0.

## Context
- docs/excel-semantics/numbers.md "Near-zero snapping", paragraph "Pickers, LET, LAMBDA and unary signs"
  (T-3049, probes/t3049-snap-under-pickers, excel-graph 2026-10-28): the snap follows the tree root of a cell
  formula, a defined-name body and a **LAMBDA body**; pickers, LET and unary signs do not pass it through.
- Known failures in eval/baseline/probes.txt: probes/t3049-snap-under-pickers.case.json Sheet1!B17
  (`=LAMBDA(x,x+0.2-0.3)(0.1)`, expected 0, engine 5.551115123125783e-17) and Sheet1!C17
  (`=LAMBDA(x,x+7*2^-52-1)(1)`, expected 0, engine 1.5543122344752192e-15). D17 (k = 8) already matches.
- The body is evaluated in `Model::call_lambda` (crates/rustsheet-engine/src/functions/logical/lambda.rs); the root
  snap lives in `Model::evaluate_formula_root` / `handle_outermost_sum` (arithmetic.rs). Reuse them, do not copy.
- Row 16 (`LET(x,0.1,x+0.2-0.3)`) must stay unsnapped: a LET body is not a root.

## Acceptance criteria
- [ ] probes/t3049-snap-under-pickers Sheet1!B17 and C17 pass and leave eval/baseline/probes.txt; no other cell
      regresses.
- [ ] numbers.md "Implemented in" names where the LAMBDA body is snapped.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LAMBDA helpers (MAP, REDUCE, BYROW, ...) and a LAMBDA bound to a defined name: unprobed; probe them first.
