---
id: T-0238
title: "Criteria: a numeric operand padded with spaces after the operator is a number"
milestone: M2
lane: functions-criteria
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A criterion whose operator is followed by a number padded with spaces (`"> 1"`, `">1 "`,
`"> 1 "`) compares numerically, as `">1"` does, in COUNTIF and in D-function criteria. A
criterion with *leading* spaces before the operator (`" >1"`, `"  > 1 "`) stays a plain-text
criterion (no change).

## Context
T-0011 probe `probes/criteria-gaps` (excel-graph, 2026-10-05), DSUM over Qty 1, 2, 4:
- C16 `">1 "` → 6, C17 `">"&" 1 "` → 6, C18 `"> 1"` → 6 (we give 0);
- C22 `COUNTIF(F2:F4,"> 1")` → 2 (we give 0);
- C15 `" >1"` → 0 and smoke/d-function-criteria J2 `"  > 1 "` → 0 already pass and must keep passing.
docs/excel-semantics/criteria.md "D-function criteria are not trimmed into an operator criterion".
Code: the shared criteria parser in `crates/rustsheet-engine/src/functions/criteria.rs`
(`build_criteria_with`; the operand after the operator is parsed as a number, which must
tolerate surrounding spaces — check how `cast_number` handles `" 1 "` before adding any trim).
Add probe cells for COUNTIF with `">1 "` and `"> 1 "` (only `"> 1"` was probed there) and for
`"= 1"` / `"<> 1"` before changing the shared parser.

## Acceptance criteria
- [ ] `probes/criteria-gaps` C16, C17, C18, C22 disappear from eval/baseline/probes.txt with no new F lines.
- [ ] Unit test for `COUNTIF(r,"> 1")` and the leading-space case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Leading-space criteria (already correct).
