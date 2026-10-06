---
id: T-1013
title: "IMAGE, STOCKHISTORY, FIELDVALUE return their offline errors (SPEC-0003 F1)"
milestone: M1
lane: functions-logic
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1010, T-1012]
spec: docs/specs/SPEC-0003-newer-error-kinds.md
---

## Goal
IMAGE, STOCKHISTORY and FIELDVALUE evaluate to Excel's offline errors (`#BLOCKED!`, `#CONNECT!`, `#FIELD!`) instead of `#NAME?`, with no I/O.

## Context
SPEC-0003 "Functions with no I/O" (rollout F1). Known failures: probes/rich-errors-image (5 cells), -stockhistory (4) and -field (4). Argument-error precedence comes from T-1012's probe.

## Acceptance criteria
- [ ] The three functions are registered with Excel's arity (including the `_xlfn.` prefix). They return the spec's error after argument errors as T-1012 found, never spill, and do no I/O.
- [ ] information.md entries for IMAGE and STOCKHISTORY have `Status: DEVIATION (decision 0014)`: reachable URLs and real tickers give errors, where Excel fetches. FIELDVALUE's entry is VERIFIED.
- [ ] The 13 rich-errors-* known failures pass and are recorded with `make eval-baseline`, with no regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Linked data types, image values, and stock data (decision 0027).
