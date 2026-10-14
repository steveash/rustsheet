---
id: T-1513
title: "Probe double-rounded products at the overflow and smallest-normal boundaries"
milestone: M2
lane: research
status: in-progress
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
- [ ] A probes-numeric case with oracle values at both boundaries; `extended_product` matches or a fix task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
