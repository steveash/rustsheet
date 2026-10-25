---
id: T-0471
title: "SUMPRODUCT over whole-column expressions takes the tailed path (SPEC-0020)"
milestone: M11
lane: perf
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0469]
---

## Goal
SUMPRODUCT over whole-column element-wise expressions (`SUMPRODUCT((A:A=x)*B:B)`) does work
proportional to the used rows, with bit-identical results (SPEC-0020 Rollout 3).

## Context
SPEC-0020: SUMPRODUCT is a second consumer of `Tailed`; each argument's element sequence must equal
today's, including the shape checks between arguments and the order of products and sums.

## Acceptance criteria
- [ ] SUMPRODUCT reads tailed arguments; equivalence tests (shapes, errors, text and booleans).
- [ ] A bench case and before/after timings in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other array consumers.
