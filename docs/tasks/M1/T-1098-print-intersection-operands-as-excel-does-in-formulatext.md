---
id: T-1098
title: "Print @ intersection operands as Excel does in FORMULATEXT"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The engine prints `=A1 @A1` and `=@A1:A3 A2:B2` exactly as written, as Excel's FORMULATEXT does.

## Context
Excel prints both as written (probes/intersection-at-formulatext H2, H4; references.md "Intersection operands must be references"). The engine prints `=A1 (@A1)` and `=@(A1:A3 A2:B2)` (T-0149 choice). Keep the round trip safe: `=A1 @A1 @A1` and `=@A2:B2 A1:A3` must still reparse to the same tree.

## Acceptance criteria
- [ ] probes/intersection-at-formulatext passes; baseline shrinks.
- [ ] Print then reparse of every intersection-at-operands formula is unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
