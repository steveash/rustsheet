---
id: T-1768
title: "Probe legacy N and T over CHOOSE, INDEX and T of a range, and a range in a scalar slot"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's legacy (non-array) `@` placement is known for N and T over a range, OFFSET and IF of one (T-1728); the neighbouring shapes are probed and recorded.

## Context
T-1728 removed the `@` only for probed shapes (`is_plain_reference`, static_analysis/legacy.rs). Unprobed, so left with the old `@`: `N(CHOOSE(1,Z1:Z2))`, `N(INDEX(Z1:Z2,1))`, `T(Z1:Z2)` and the T versions of the OFFSET/IF cases, and a range in a scalar slot such as `N(IF(A1:A2,Z1,Z2))`, `N(OFFSET(Z1,A1:A2,0))`, `N(INDEX(Z1:Z2,A1:A2))`, `N(CHOOSE(A1:A2,Z1,Z2))`. Probe each legacy with FORMULATEXT (like probes/legacy-n-t-reference-args), record in references.md "Legacy N and T over a reference-returning call", and file engine tasks for any that differ from ours.

## Acceptance criteria
- [ ] A probe case with excel-graph values covers each shape above, N and T, with FORMULATEXT.
- [ ] references.md records each result; engine follow-up tasks filed for mismatches.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/legacy-n-t-reference-args-2 (excel-graph, 2026-10-18): N and T over CHOOSE/INDEX of a reference, T of a range, and a range in a scalar slot, with FORMULATEXT. references.md records each result. Only N/T over CHOOSE or INDEX of a reference differ from the engine (T-1908). Eval: new known failures for those cells in the probes baseline.
