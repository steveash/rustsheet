---
id: T-0109
title: "Generate and oracle-fill suite gen-m1 (operators and references only)"
milestone: M1
lane: harness
status: blocked
priority: P1
size: S
depends_on: [T-0004, T-0117]
---

## Goal
≥ 3,000 oracle-backed cells exercising M1 semantics.

## Context
`rustsheet-gen --milestone M1`.

## Acceptance criteria
- [ ] Suite gen-m1 with sidecars committed; gate = true once filled; baseline recorded; gap tasks filed by gap-triager.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
Needs the Excel oracle (T-0004) — owner must provide Graph credentials (docs/architecture/oracle.md).
