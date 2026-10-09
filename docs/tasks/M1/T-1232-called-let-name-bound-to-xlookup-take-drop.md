---
id: T-1232
title: "Called LET name bound to XLOOKUP, TAKE, DROP or TRIMRANGE is #REF! (follow-up of T-1224)"
milestone: M1
lane: functions-logic
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A LET name bound to XLOOKUP, TAKE, DROP or TRIMRANGE and called like a function gives `#REF!` as Excel does.

## Context
probes/let-name-call-forms H7 (XLOOKUP, single cell), H8 (TAKE, one cell), H9 (DROP) and H11 (TRIMRANGE) (excel-graph 2026-10-09, T-1224) are `#REF!`; the engine gives `#NAME?` for XLOOKUP/TAKE (Unsupported) and `#SPILL!` for DROP/TRIMRANGE. OFFSET already matches. Whether these bind a reference (ISREF) is not probed. See docs/excel-semantics/references.md "Calling a parenthesized operand"; code in functions/logical/lambda.rs `call_variable` and let.rs `bind_argument`.

## Acceptance criteria
- [x] the four cells pass and leave eval/baseline
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`Model::bind_argument` binds XLOOKUP, TAKE, DROP and TRIMRANGE through `evaluate_picked_reference_arg`, so a called LET name is `#REF!`. Eval delta: probes known failures 563 -> 559 (4 fixed: let-name-call-forms H7, H8, H9, H11), 0 regressions.
