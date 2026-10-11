---
id: T-1169
title: "Debug-build stack overflow at about 126 nested parentheses in a cell formula"
milestone: M1
lane: engine
status: in-progress
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
- [ ] A test with 256 nested parentheses (and 300) returns a value or an error without overflowing the default test thread stack.
- [ ] The recursion that overflows is bounded or made iterative; a DEVIATION entry if Excel behavior differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
