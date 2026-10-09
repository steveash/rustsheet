---
id: T-1159
title: "Probe 3-D operands in unions: right operand, bare union, error precedence, intersection (follow-up of T-1070)"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's answers for 3-D operands in unions are recorded for the cells T-1070 could only guess.

## Context
T-1070 implemented `#NAME?` for a 3-D operand inside a union from one oracle cell (`SUM((Sheet1:Sheet2!A1,B2))`, probes/multiarea-3d-ref). Engine choices without evidence, listed UNVERIFIED in docs/excel-semantics/references.md ("Multi-area references in functions"): the right operand `SUM((A1,Sheet1:Sheet2!B2))`; the bare value context `=(Sheet1:Sheet2!A1,B2)`; precedence against another operand's error `SUM((1/0,Sheet1:Sheet2!A1))`; a 3-D operand of an intersection. Use the `excel-probe` skill.

## Acceptance criteria
- [x] A probe case holds each formula above, with oracle values.
- [x] references.md records the answers (UNVERIFIED marks removed or the engine task filed).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing engine behavior (file a task if an oracle value differs).

## Outcome
Eight probe cases (multiarea-3d-union-h1..h8) with excel-graph values: 3-D union operands are `#NAME?` on either side, in a bare context, under AREAS and ahead of another operand's error; 3-D operands of an intersection are `#VALUE!`. The engine already matched all eight; references.md records the answers and drops the UNVERIFIED marks. Eval delta: +8 cases, +10 cells, all passing, no known failures added (8 coverage lines in the probes baseline).
