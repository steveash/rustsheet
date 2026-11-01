---
id: T-3206
title: "Probe stored even-arity LET shapes and multi-LF intersection separators (T-3169 inferred rules)"
milestone: M2
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior for the shapes T-3169 generalised beyond its oracle evidence is known and recorded.

## Context
T-3169 appends the last name to every stored even-arity LET and keeps any LF-holding intersection separator whole, from one probe each (probes/t2949-stored-11, typed-4/5, stored-14). Unprobed: stored `LET(a,1)`, `LET(a,1,b,2,c,3,d,4)`, a non-name second-to-last argument, several LFs or LF plus U+3000 in an intersection separator. See docs/excel-semantics/syntax.md "Remaining whitespace shapes (T-2949)"; code in `result_of_even_let` and `Parser::parse_intersection`.

## Acceptance criteria
- [ ] Probe cases for each shape above have oracle values; syntax.md states the verified rules; any mismatch gets an engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
