---
id: T-1103
title: "Probe whether Excel loads _xlfn.SINGLE as an intersection operand"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence whether Excel loads `_xlfn.SINGLE(...)` as an intersection operand.

## Context
From the T-1098 reviews: the engine exports `A1 @A1:A3` as `A1 _xlfn.SINGLE(A1:A3)` (previously `A1 (_xlfn.SINGLE(A1:A3))`); SINGLE is not in the probed list of accepted operands in references.md. Use the `excel-probe` skill: one workbook per form (a refused workbook gives `FileCorruptTryRepair`; record, do not commit).

## Acceptance criteria
- [x] Probe with oracle sidecar (or recorded refusal) for `=A1 _xlfn.SINGLE(A1:A3)`; references.md updated; an engine mismatch is filed as an engine task.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Excel loads `_xlfn.SINGLE(...)` as an intersection operand (probes/intersection-single-operands, excel-graph): 1, 2, `#VALUE!`, 1. The engine already matches all four cells, so no engine task. references.md updated. Eval delta: +4 probe cells passing, no baseline change.
