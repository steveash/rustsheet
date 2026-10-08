---
id: T-1159
title: "Probe 3-D operands in unions: right operand, bare union, error precedence, intersection (follow-up of T-1070)"
milestone: M1
lane: research
status: todo
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
- [ ] A probe case holds each formula above, with oracle values.
- [ ] references.md records the answers (UNVERIFIED marks removed or the engine task filed).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing engine behavior (file a task if an oracle value differs).
