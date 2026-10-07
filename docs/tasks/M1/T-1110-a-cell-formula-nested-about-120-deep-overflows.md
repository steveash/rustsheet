---
id: T-1110
title: "A cell formula nested about 120 deep overflows a debug-build test thread"
milestone: M1
lane: engine
status: todo
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
- [ ] A unit test with a formula at the parser's nesting limit evaluates without overflow in a
  debug build (the result may be an error when the stack budget runs out; record it as a
  DEVIATION if it differs from Excel).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Defined-name nesting (T-0158 bounds it).
