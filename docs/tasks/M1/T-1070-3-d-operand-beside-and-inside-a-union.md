---
id: T-1070
title: "3-D operand beside and inside a union: SUM(3-D,(A1,B2)) works, union containing 3-D is #NAME? (SPEC-0004 row 13)"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
A 3-D operand beside a union works, and inside a union gives #NAME?.

## Context
Oracle (probes/multiarea-3d-ref): SUM(Sheet1:Sheet2!A1,(A1,B2))=107 (3-D argument plus the union), SUM((Sheet1:Sheet2!A1,B2))=#NAME? (SPEC-0004 planned #VALUE!). Engine gives #VALUE! for both. The first needs T-1062's migration; the second is a union-building rule (`Areas::new`/`union_areas`). Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] Both cells of multiarea-3d-ref pass
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' union support (their own tasks under SPEC-0004).
