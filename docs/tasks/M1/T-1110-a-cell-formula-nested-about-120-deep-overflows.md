---
id: T-1110
title: "A cell formula nested about 120 deep overflows a debug-build test thread"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Evaluating a single cell whose formula nests as deep as the parser allows never overflows the
native stack, in debug builds and on 2 MB threads included (directive 5).

## Context
Found in T-0158: `=ABS(ABS(...1...))` nested 120 deep in one cell aborts a debug-build test
thread with "has overflowed its stack" (about 17 KB of stack per nesting level). The release
build passes. The parser allows MAX_FORMULA_DEPTH = 256 (expressions/parser/depth.rs), and the
evaluator's stack budget (evaluation.rs, `max_stack`, 256 KiB) is checked only between cells,
not within one formula's AST walk. Reproduce: a unit test that sets A1 to that formula and
evaluates, run with `cargo test` (debug).

## Acceptance criteria
- [x] A unit test with a formula at the parser's nesting limit evaluates without overflow in a
  debug build (the result may be an error when the stack budget runs out; record it as a
  DEVIATION if it differs from Excel).
- [x] `make check` green; reviewers approve.

## Out of scope
- Defined-name nesting (T-0158 bounds it).

## Outcome
The overflow was mostly in the parser, not the evaluator: in a debug build each `ABS(` level took
~22 KB of parser stack (`parse_operand`'s ~400-line match alone ~11 KB) and ~13 KB of evaluator
stack (`evaluate_function`'s ~500-arm match ~6 KB). `parse_operand`'s arms moved to
`expressions/parser/operands.rs` (one method per token kind, the call path split from names),
`parse_chain` keeps its link loop in `parse_chain_links`, and `evaluate_function` picks a function
pointer and calls it once. Debug stack for a cell nested to the parser's limit (127 `ABS(`):
~2.9 MB -> ~1.55 MB, so it fits a 2 MiB thread with no result change (no DEVIATION).
`test_nested_formula_stack` checks six nesting shapes at their own limits on a 2 MiB thread (it
aborts with a stack overflow on main). Eval delta: none (0 regressions).
