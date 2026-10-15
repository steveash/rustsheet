---
id: T-0958
title: "T-0918 code follow-ups: is_finite in rept_element, fold the two R1C1 reference parsers"
milestone: M9
lane: quality
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`rept_element` checks its count is finite explicitly, and one R1C1 reference parser serves both callers.

## Context
T-0918 review follow-up (carried in T-0920): `rept_element` in
crates/rustsheet-engine/src/functions/text/common.rs relies on implicit range checks for a NaN or
infinite count; `parse_reference_r1c1` in expressions/utils/mod.rs and the R1C1 parser added for
INDIRECT duplicate logic (CLAUDE.md directive 3).

## Acceptance criteria
- [ ] Explicit `is_finite` check with a unit test; one R1C1 parser, no behavior change (eval unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
