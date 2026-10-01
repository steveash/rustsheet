---
id: T-0203
title: "Generate and oracle-fill gen-m2 (≥ 10k cells)"
milestone: M2
lane: harness
status: blocked
priority: P1
size: S
depends_on: [T-0004, T-0202]
---

## Goal
Oracle-backed coverage for all M2 functions.

## Context
-

## Acceptance criteria
- [ ] gen-m2 ≥ 10,000 oracle cells; gate = true; baseline recorded; gap tasks filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Blocked because
Needs the Excel oracle (T-0004) — owner must provide Graph credentials (docs/architecture/oracle.md).
