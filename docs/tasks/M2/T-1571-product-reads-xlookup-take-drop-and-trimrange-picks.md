---
id: T-1571
title: "PRODUCT reads an XLOOKUP pick as a reference like SUM"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
PRODUCT over a cell picked by XLOOKUP skips a text cell like SUM does (`PRODUCT(XLOOKUP(1,A1:A2,B1:B2),2)` is 2).

## Context
probes/aggregate-picked-direct-a-variants D1 (ours: #VALUE!; the TAKE and DROP cells D3 and D5 already match and TRIMRANGE is unprobed). PRODUCT reads with `evaluate_reference_arg` in product.rs; SPEC-0009 row PRODUCT. Apply after T-1474/T-1476 as the spec says.

## Acceptance criteria
- [ ] The probe cells named in Context pass (`rustsheet-eval show`), no new known failures; the SPEC-0009 row is updated from DIVERGES to matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cases the probe did not cover.
