---
id: T-0008
title: "Generator P1: numeric-rule probe batches"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
depends_on: []
---

## Goal
Generate dense probe workbooks that pin down Excel's floating-point rules (near-zero snapping in +/-, comparison tolerance, 15-digit literal parsing, number→text).

## Context
Design: docs/architecture/synthetic-generator.md §5.2 and §6.1; docs/excel-semantics/numbers.md lists hypotheses.

## Acceptance criteria
- [ ] `rustsheet-gen --policy numeric-rules` (or equivalent) writes ≥ 2,000 probe cells into suite `probes-numeric` (pending oracle).
- [ ] Each cell has a tag describing the hypothesis it tests; deterministic by seed.
- [ ] pytest coverage for the policy.
- [ ] `make check` green; reviewers approve.

## Out of scope
Running the oracle (T-0004).
