---
id: T-1546
title: "Probe lifted INDEX over a cached external range (cached, whole-row, out-of-range elements)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's value for `INDEX([1]Uses!$A$1:$B$2,{1,2},1)` and `{0,2},1` over a cached external range is
known, and the engine matches it.

## Context
T-1537 lifts INDEX over index arrays into an external reference; only the no-cache case has oracle
evidence (probes/external-ref-no-link-ifchoose H1, I1). Cached elements, whole-row/column
elements (top-left kept), out-of-range elements and a nested INDEX-as-reference are UNVERIFIED
(docs/excel-semantics/references.md). Use the `excel-probe` skill; extend
fixtures/external-link-cached-ranges.

## Acceptance criteria
- [ ] Oracle-backed cases cover the four UNVERIFIED behaviours; the engine passes or the gaps are filed.
- [ ] references.md drops the UNVERIFIED marks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
