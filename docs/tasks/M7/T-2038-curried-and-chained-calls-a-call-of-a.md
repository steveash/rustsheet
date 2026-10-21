---
id: T-2038
title: "Curried and chained calls: a call of a call, LAMBDA(a,LAMBDA(b,a*b))(3)(4) and INDEX(s,1,3)(0)"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A call whose callee is itself a call evaluates like Excel: `=LAMBDA(a,LAMBDA(b,a*b))(3)(4)` is 12 and
`=LAMBDA(a,LAMBDA(b,LAMBDA(c,a*100+b*10+c)))(1)(2)(3)` is 123, instead of `#ERROR!` (a parse error).

## Context
Found by T-0439. probes/lambda-escaping-closures H8 and H10 (excel-graph) give 12 and 123; the engine
does not parse a second `(...)` after a LAMBDA call (`Parser::parse_call` in
expressions/parser/lambda.rs returns after one call). H3, `INDEX(s,1,3)(0)` over a SCAN of LAMBDAs, also
needs T-2039. Mind the intersection rule: `(A1) (A2)` with a space is an intersection
(docs/excel-semantics/references.md "Calling a parenthesized operand"); probe whether `f(1) (2)` with a
space and a function call result `INDEX(...)(0)` are calls before extending the grammar.

## Acceptance criteria
- [ ] probes/lambda-escaping-closures H8 and H10 pass; any new grammar case is probed first.
- [ ] FORMULATEXT / round trip of a chained call keeps its text.
- [ ] `make check` green; reviewers approve.

## Out of scope
- LAMBDA values in arrays (T-2039).
