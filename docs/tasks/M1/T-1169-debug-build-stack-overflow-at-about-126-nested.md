---
id: T-1169
title: "Debug-build stack overflow at about 126 nested parentheses in a cell formula"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A cell formula with up to MAX_FORMULA_DEPTH (256) nested parentheses never aborts the process: it parses, evaluates and prints, or becomes a ParseError, in debug and release builds.

## Context
Found by a reviewer of T-1161: `=` + N x `(` + `0.1+0.2-0.3` + N x `)` set through `model._set` then `evaluate` aborts with a stack overflow in a debug test build once N reaches about 126, on main and on the T-1161 branch alike (N=50 works). The overflow site (parser frame size, evaluator or stringify recursion on nested nodes) is not yet found. Directive 5: no unbounded recursion.

## Acceptance criteria
- [x] A test with 256 nested parentheses (and 300) returns a value or an error without overflowing the default test thread stack.
- [x] The recursion that overflows is bounded or made iterative; a DEVIATION entry if Excel behavior differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The overflow no longer reproduces on main: T-1110 already sized the parser, evaluator and printer
recursion so a formula at `MAX_FORMULA_DEPTH` runs on a 2 MiB debug test thread. The reported shape
(`=` + N `(` + `0.1+0.2-0.3+B1` + N `)`) is accepted up to N = 126, because the inner chain costs one
level, and refused as "Formula is nested too deeply" from 127 up (256 and 300 included).
`test_nested_formula_stack.rs` now pins that on the default test thread: parse, evaluate, print, a row
insert and a sheet rename at 125 and 126 levels, and the refusal at 127, 256 and 300. syntax.md gives
the 126 limit for a chain. The existing DEVIATION entry ("Formula nesting depth cap") covers the
difference from Excel. Eval delta: none (unit tests only).
