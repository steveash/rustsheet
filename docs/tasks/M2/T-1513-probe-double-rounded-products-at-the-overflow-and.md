---
id: T-1513
title: "Probe double-rounded products at the overflow and smallest-normal boundaries"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Probe in Excel what `*` and PRODUCT return when the double-rounded product lands on DBL_MAX-to-overflow or on the smallest normal (2^-1022).

## Context
T-1498 falls back to the singly-rounded product when the second rounding leaves the normal range (`arithmetic::extended_product`); no oracle case reaches these boundaries. Reviewers flagged it as an evidence gap.

## Acceptance criteria
- [x] A probes-numeric case with oracle values at both boundaries; `extended_product` matches or a fix task is filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added `probes-numeric/double-rounding-range-edges` (40 cells, excel-graph). Excel double-rounds at both
edges: a product just under the DBL_MAX/2^1024 midpoint is `#NUM!`, and one just under the
largest-subnormal/2^-1022 midpoint is 2^-1022 (tie on the subnormal grid). `extended_product` now
matches (the case passes 40/40; probes-numeric 8251 -> 8265 passing cells). Quirk recorded in
`docs/excel-semantics/numbers.md`. Test module moved to `arithmetic/power_tests.rs` to stay under the
1000-line file limit.
