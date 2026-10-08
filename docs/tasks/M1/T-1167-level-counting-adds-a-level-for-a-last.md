---
id: T-1167
title: "@(...) level counting adds a level for a last inner group (same end-only check as T-1161 had)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`@(...)` prints exactly the parenthesis levels written, with a regression test for each shape.

## Context
A reviewer of T-1161 saw that the `at_group` level counting in `parse_parenthesized` (parser/reference_operators.rs) uses an end-only check, which T-1161 had to fix for whole-formula groups: an inner group ending right before the outer `)` counts as a nested level even if it does not start the outer content. `@(A1+(B1))` probably prints an extra level. Unverified: reproduce first.

## Acceptance criteria
- [ ] A test reproduces the extra level (or shows there is none) for `@(A1+(B1))`, `@((A1)+(B1))`, `@((A1))`; the counting matches what Excel keeps (docs/excel-semantics/references.md, `@` levels).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
