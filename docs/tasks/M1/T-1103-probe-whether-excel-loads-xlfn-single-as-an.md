---
id: T-1103
title: "Probe whether Excel loads _xlfn.SINGLE as an intersection operand"
milestone: M1
lane: research
status: in-progress
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
- [ ] Probe with oracle sidecar (or recorded refusal) for `=A1 _xlfn.SINGLE(A1:A3)`; references.md updated; an engine mismatch is filed as an engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
