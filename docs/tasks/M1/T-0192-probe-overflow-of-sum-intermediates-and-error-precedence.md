---
id: T-0192
title: "Probe overflow of SUM intermediates and error precedence, SUMPRODUCT, AVERAGE, PRODUCT, SUMSQ, SUBTOTAL, AGGREGATE, PERCENTOF"
milestone: M1
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for aggregate overflow: SUM with an intermediate overflow, SUM error precedence (`SUM(9E+307,9E+307,NA())`), SUMPRODUCT, AVERAGE, PRODUCT, SUMSQ, SUBTOTAL, AGGREGATE and PERCENTOF.

## Context
Follow-up to T-0185 (fidelity review). docs/excel-semantics/numbers.md lists these overflow cases as UNVERIFIED. Use the `excel-probe` skill; extend probes/arith-overflow or add a case.

## Acceptance criteria
- [ ] Oracle values recorded for each function above; numbers.md updated with VERIFIED results; implementation tasks filed for any that differ.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
