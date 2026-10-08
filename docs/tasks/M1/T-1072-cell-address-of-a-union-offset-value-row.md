---
id: T-1072
title: "CELL address of a union, OFFSET #VALUE!, ROW #REF! (SPEC-0004 row 18)"
milestone: M1
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
CELL("address",union)="$A$1, $B$2", OFFSET(union,1,1)=#VALUE!, ROW(union)=#REF!.

## Context
Oracle (probes/multiarea-cellfn); SPEC-0004 planned #VALUE! for CELL and ROW, both differ. Probe other CELL info types in the same PR if cheap (excel-probe skill). Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] The three cells of multiarea-cellfn pass; other info types recorded in references.md
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' union support (their own tasks under SPEC-0004).
