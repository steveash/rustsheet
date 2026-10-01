---
id: T-0110
title: "Near-zero snapping in final addition/subtraction"
milestone: M1
lane: engine
status: blocked
priority: P2
size: S
depends_on: [T-0004]
---

## Goal
Implement Excel's rule once VERIFIED in docs/excel-semantics/numbers.md.

## Context
Probe cells A2–A4, A15.

## Acceptance criteria
- [ ] Probe cells pass; rule documented with evidence; implemented in one place.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
Needs the Excel oracle (T-0004) — owner must provide Graph credentials (docs/architecture/oracle.md).
