---
id: T-0133
title: "Spec: multi-area references (union) in the engine"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0134]
---

## Goal
A spec in `docs/specs/` that settles how the engine represents a multi-area reference, so unions
can reach functions that take them (`SUM((A1,B2))`, `AREAS`, `INDEX(…,area_num)`) without a
redesign.

## Context
T-0134 implements only what gen-m1 measures: a union in a value context gives `#VALUE!`, and an
intersection that cuts it down to one cell gives that cell. It deliberately leaves out a general
multi-area value. CLAUDE.md directive 4 asks for a spec before that architectural step. The spec
should decide:
- the `CalcResult` (or reference) representation of several areas, possibly on different rows,
  columns or sheets;
- how functions declare that they accept multi-area references;
- the lexing rule for a union comma versus an argument separator, including comma-decimal locales;
- stringify round-trip;
- which Excel behaviour needs an excel-probe first (AREAS of a nested union, a cross-sheet union),
  at least `SUM((A1,B2))`, `AREAS(((A1,B2),C3))`, `INDEX((A1:B2,C3:D4),1,1,2)`,
  `LET(x,(A1,B2),x A1)` and `IF(TRUE,(A1,B2)) A1`;
- defined names whose body is a union (`_xlnm.Print_Area` is usually `Sheet1!$A$1,Sheet1!$B$2`),
  which do not parse today;
- whether `add_implicit_intersection` (static_analysis.rs) should wrap `OpUnionKind`, which it
  leaves unwrapped unlike Range and Intersect nodes.

Today a union passed to a function is `#VALUE!`, so IFERROR silently takes its fallback for one
(T-0134); the spec should say what changes for such formulas. Edge cases of the operator itself
are probed in T-0187.

## Acceptance criteria
- [x] `docs/specs/SPEC-XXXX-multi-area-references.md` is merged, with the representation, function
  contract, lexing rule, probes needed and follow-up implementation tasks filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing it (the follow-up tasks do that).

## Outcome
docs/specs/SPEC-0004-multi-area-references.md. A multi-area reference is `Areas` (a boxed slice of
ranges, 2..=`MAX_AREAS`), kept out of `CalcResult` on purpose: a new variant would reach every
`_ =>` arm and make `ISERROR((A1,B2))` FALSE. Functions opt in through one reference-context
evaluator, `evaluate_reference_arg -> RefArg`, built on the existing `reference_areas`. The value
path still gives `#VALUE!`. LET bindings can hold `Areas`, IF/CHOOSE/INDEX pass them through, and
multi-area name bodies (Print_Area) parse with the comma as the union at the top level. The lexing
rule and printing are unchanged, and `add_implicit_intersection` keeps unions unwrapped. The spec
has an 18-row probe table. Rollout: T-1061 (probes), T-1062 (type, evaluator, aggregates), T-1063
(AREAS, INDEX area_num), T-1064 (LET, IF, CHOOSE), T-1065 (names). references.md now points to
the spec. Eval: no change (docs only).
