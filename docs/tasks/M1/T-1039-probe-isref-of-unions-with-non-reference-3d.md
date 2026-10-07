---
id: T-1039
title: "Probe ISREF of unions with non-reference, 3D and empty-intersection operands"
milestone: M1
lane: harness
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's ISREF is known for unions with a non-reference, 3D or empty-intersection operand, and the engine matches or the gap is baselined.

## Context
Follow-up of T-0194 review. `Model::union_is_reference` returns FALSE for any area-list error, so `ISREF((-A1,B1))`, `ISREF((A1 B2,C1))` and `ISREF((Sheet1:Sheet2!A1,B1))` changed from TRUE to FALSE without oracle data; `ISREF(A1 B2)` alone is always TRUE (information/mod.rs). Probe them with excel-probe, record in references.md, fix the engine where it differs.

## Acceptance criteria
- [ ] A probe records Excel's ISREF for `(-A1,B1)`, `(A1 B2,C1)`, `A1 B2` and a 3D operand union; engine matches or gap baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
