---
id: T-0190
title: "Use paren_if in the OpPowerKind arm of stringify"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The `OpPowerKind` arm of `stringify` wraps operands via `paren_if` and shared predicates instead of two hand-written `match` blocks.

## Context
Follow-up to T-0184 (quality review MINOR). `crates/rustsheet-engine/src/expressions/parser/stringify.rs`. Pure refactor; printed text must not change.

## Acceptance criteria
- [x] Existing stringify round-trip tests pass unchanged; no printed formula changes.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The `OpPowerKind` arm of `stringify` uses `paren_if` and one `is_atom` predicate instead of two hand-written `match` blocks (-41 lines). Printed text unchanged; existing round-trip tests pass. No eval delta.
