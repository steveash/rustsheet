---
id: T-0192
title: "Probe overflow of SUM intermediates and error precedence, SUMPRODUCT, AVERAGE, PRODUCT, SUMSQ, SUBTOTAL, AGGREGATE, PERCENTOF"
milestone: M1
lane: research
status: done
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
- [x] Oracle values recorded for each function above; numbers.md updated with VERIFIED results; implementation tasks filed for any that differ.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe aggregate-overflow (48 formulas, excel-graph). SUM overflows at the intermediate point; error precedence is first error in argument order and beats overflow; SUMPRODUCT, AVERAGE, PRODUCT, SUMSQ, SUBTOTAL, AGGREGATE and 9 more overflow to #NUM!. PERCENTOF is #NAME? in the oracle, so it was left out of the probe and stays unverified. Recorded in numbers.md; engine tasks T-0990, T-0991, T-0992 and T-0993 (AGGREGATE). Eval delta: new-coverage failures only.
