---
id: T-1496
title: "Probe whether *, function-internal divisions and other ops round twice like /"
milestone: M2
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for which numeric operations besides the operator `/` round twice (to 64 bits,
then to a double), as `/` does (numbers.md "Division rounds twice").

## Context
T-0221 found that `=1/r` is double-rounded in Excel for 8 of 8 values r picked so that the two
roundings differ (probes-numeric/power-reciprocal-rounding). Use the same construction
(`1+k*2^-52` literals, k picked by simulating both roundings; see that case's description) to
probe: `*` (`a*b` with both full-significand), AVERAGE of two cells (sum/count), MOD, QUOTIENT,
`x/y` with neither 1, a percentage (`x%`), SQRT, and division inside a function such as
`=SUMPRODUCT(1/A1:A3)`. One probe case, `expected: {}`, oracle excel-graph. Record the findings
in numbers.md "Division rounds twice" and file an engine task for each operation that also rounds
twice (the helper is `arithmetic::extended_quotient`).

## Acceptance criteria
- [ ] Probe case with oracle values; numbers.md updated with the result for each operation.
- [ ] Follow-up implementation task(s) filed for operations that round twice.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing double rounding for any newly found operation (the follow-up tasks).
