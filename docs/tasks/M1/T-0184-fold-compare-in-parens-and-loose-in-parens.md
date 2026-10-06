---
id: T-0184
title: "Fold compare_in_parens and loose_in_parens in stringify into one predicate"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`stringify` has one shared "needs parentheses as an arithmetic operand" predicate instead of `compare_in_parens`, `loose_in_parens` and hand-written `match` arms in the Sum and Product arms.

## Context
Follow-up to T-0157 (architecture and quality review MINOR findings). `crates/rustsheet-engine/src/expressions/parser/stringify.rs`. Pure refactor; printed text must not change.

## Acceptance criteria
- [ ] Existing stringify round-trip tests pass unchanged; no printed formula changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
