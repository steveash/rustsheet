---
id: T-1069
title: "COUNTBLANK, ROWS and COLUMNS of a union: #VALUE!, #REF!, #REF! (SPEC-0004 row 5)"
milestone: M1
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
COUNTBLANK, ROWS and COLUMNS of a union give #VALUE!, #REF!, #REF!.

## Context
Oracle (probes/multiarea-count): COUNTBLANK((A1,B2))=#VALUE!, ROWS=#REF!, COLUMNS=#REF!. The engine gives 0, #VALUE!, #VALUE!. Plain-engine fix; may need no `Areas` (a union where one reference is needed). Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] The three cells of multiarea-count pass
- [ ] `make check` green; reviewers approve.

## Out of scope
-
