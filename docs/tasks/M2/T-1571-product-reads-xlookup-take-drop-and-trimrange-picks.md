---
id: T-1571
title: "PRODUCT reads XLOOKUP, TAKE, DROP and TRIMRANGE picks as references like SUM"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
PRODUCT over a cell picked by XLOOKUP, TAKE or DROP skips a text cell like SUM does (`PRODUCT(XLOOKUP(1,A1:A2,B1:B2),2)` is 2).

## Context
probes/aggregate-picked-direct-a-variants D1, D3, D5 (ours: #VALUE!). PRODUCT reads with `evaluate_reference_arg` in product.rs; SPEC-0009 row PRODUCT. Apply after T-1474/T-1476 as the spec says.

## Acceptance criteria
- [ ] The probe cells named in Context pass (`rustsheet-eval show`), no new known failures; the SPEC-0009 row is updated from DIVERGES to matched.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
