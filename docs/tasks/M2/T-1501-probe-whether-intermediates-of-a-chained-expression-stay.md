---
id: T-1501
title: "Probe whether intermediates of a chained expression stay at 64 bits (a*b+c, SQRT(a*b))"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether a chained expression keeps a 64-bit intermediate or rounds to a double between operators.

## Context
T-1496 showed each of `*`, `+`, `-`, SQRT rounds twice taken alone (numbers.md "Division rounds twice"), with exact operands. It did not test `=a*b+c`, `=a*b/c` or `=SQRT(a*b)`, where an x87 intermediate could stay at 64 bits. Build operands by exact simulation as in probes-numeric/double-rounding-ops; one probe case, `expected: {}`, oracle excel-graph.

## Acceptance criteria
- [ ] Probe case with oracle values; numbers.md records, per chained form, which intermediate precision Excel uses; follow-up engine task filed if it differs from rounding after each operator.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes-numeric/double-rounding-chained` (54 cells, excel-graph): in `a*b+c`, `a*b-c`, `(a+b)*c`,
`a*b*c`, `a*b/c`, `a/b*c`, `SQRT(a*b)`, `SQRT(a+b)` each operator's result is rounded twice to a
double before the next operator (48 of 48 discriminating cells; SUM(a*b,c) control agrees). No
follow-up engine task beyond T-1498, T-1499 and T-1500: once each operator rounds twice, chains
follow. The ungated probes-numeric suite has no baseline change.
