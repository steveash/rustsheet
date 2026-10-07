---
id: T-1074
title: "Probe FORMULATEXT of a union in a legacy (non-array) formula (SPEC-0004 row 16)"
milestone: M1
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what FORMULATEXT shows for a union written as a legacy (non-array) formula: does Excel add `@`?

## Context
T-1061 could not probe it: `spec2xlsx.py` writes every formula as a dynamic-array formula (probes/multiarea-at shows `=(A1,B2)` and `=SUM((A1,B2))` without `@` only for that kind). Add a spec2xlsx option (a per-cell or per-case flag) that writes a plain `<f>` with no `cm`, probe `=(A1,B2)`, `=SUM((A1,B2))` and `=@(A1,B2)` with FORMULATEXT, record the result in references.md "Multi-area references in functions", and file an engine follow-up adding `@` to the printed formula if Excel does (SPEC-0004 "add_implicit_intersection and OpUnionKind").

## Acceptance criteria
- [ ] Legacy-formula probe case with oracle values kept; references.md and SPEC-0004 row 16 updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
