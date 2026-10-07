---
id: T-1071
title: "SUBTOTAL accepts a union (SPEC-0004 row 17)"
milestone: M1
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
SUBTOTAL(9,..) and SUBTOTAL(109,..) accept a union.

## Context
Oracle (probes/multiarea-subtotal): both give 6 for (A1,B2) with A1=1, B2=5. Engine gives #VALUE!. Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] Both cells of multiarea-subtotal pass
- [ ] `make check` green; reviewers approve.

## Out of scope
-
