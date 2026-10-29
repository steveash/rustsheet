---
id: T-3169
title: "Match Excel for the T-2949 whitespace shapes (LAMBDA and LET names, intersection LF, leading TAB/LF in stored text)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT prints the shapes probed in T-2949 as Excel does.

## Context
Evidence and rules: docs/excel-semantics/syntax.md "Remaining whitespace shapes (T-2949)"; cases probes/t2949-typed-* and t2949-stored-*. LAMBDA and LET name shapes already match. Fix what fails: an intersection separator's line feed is kept (typed-4, typed-5, stored-14); a stored leading LF is dropped (stored-5); a stored even-arity LET `=LET( a ,1, b ,a+1)` loads, prints `=LET( a,1, b,a+1,b)` and returns 2 (stored-11).

## Acceptance criteria
- [ ] Every t2949 probe cell passes (or its difference is recorded as a DEVIATION).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
