---
id: T-1546
title: "Probe lifted INDEX over a cached external range (cached, whole-row, out-of-range elements)"
milestone: M2
lane: engine
status: done
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
- [x] Oracle-backed cases cover the four UNVERIFIED behaviours; the engine passes or the gaps are filed.
- [x] references.md drops the UNVERIFIED marks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Extended fixtures/external-link-cached-ranges with Refs!B83:D92 (Excel for the web values). The four behaviours
(cached elements, whole-row/column elements, out-of-range elements, cached error) match in the engine. Excel
treats the lifted INDEX as a reference to its first element, so SUM/ROWS/ISREF/nested INDEX over it differ
from ours (Refs!B89:B92); filed T-1554. Eval delta: 17 new cells, 13 pass, 4 known failures (new coverage).
