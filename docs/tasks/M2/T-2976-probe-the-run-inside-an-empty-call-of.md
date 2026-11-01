---
id: T-2976
title: "Probe the run inside an empty call of a LAMBDA or a name (=f( )) and keep it if Excel does"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Evidence for whether Excel keeps the run inside an empty call of a LET-bound LAMBDA, a defined-name LAMBDA or an inline LAMBDA (`=LET(f,LAMBDA(1),f( ))`, `=LAMBDA(1)( )`), and ours matches.

## Context
T-2966 keeps the run inside an empty built-in call (`=NOW( )`, probes/t2966-empty-call-runs) as one spaced empty
argument that `strip_spaces` removes (`Node::with_empty_call_run`, expressions/parser/spacing.rs). Calls of a name
(`NamedFunctionKind`) and of a LAMBDA (`LambdaCallKind`) still drop it, unprobed (syntax.md "Whitespace beside a
missing argument is kept", Ours). Use the `excel-probe` skill with `entered` cells and FORMULATEXT beside them; put
each shape Graph may refuse in its own case.

## Acceptance criteria
- [ ] A probe case answers the shapes above; syntax.md records the result.
- [ ] If Excel keeps the run, ours prints it back, and `strip_spaces` still gives the call no argument (extend
      `is_empty_call_run` to those call nodes).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Built-in calls (T-2966, done).
