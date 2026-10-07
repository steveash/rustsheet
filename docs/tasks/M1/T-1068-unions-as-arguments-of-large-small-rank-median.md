---
id: T-1068
title: "Unions as arguments of LARGE, SMALL, RANK, MEDIAN, STDEV.S and FREQUENCY (SPEC-0004 rows 3-4)"
milestone: M1
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1062]
---

## Goal
LARGE, SMALL, RANK, MEDIAN, STDEV.S and FREQUENCY accept a union argument, giving what Excel gives.

## Context
Oracle (probes/multiarea-rank, multiarea-cond): LARGE((A1,B2,C3),2)=5, SMALL(..,2)=5, RANK(5,(A1,B2,C3))=2, MEDIAN=5, STDEV.S=3.0550504633 (A1=1,B2=5,C3=7); SUM(FREQUENCY((A1,B2),3))=2. LARGE/SMALL/RANK give #N/IMPL! today, FREQUENCY #VALUE!. SUMPRODUCT, SUMIF and COUNTIF of a union are #VALUE! in Excel (already so; keep). Spec: docs/specs/SPEC-0004-multi-area-references.md; evidence: docs/excel-semantics/references.md "Multi-area references in functions".

## Acceptance criteria
- [ ] The cells of multiarea-rank and multiarea-cond pass; the baseline shrinks
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other functions' union support (their own tasks under SPEC-0004).
