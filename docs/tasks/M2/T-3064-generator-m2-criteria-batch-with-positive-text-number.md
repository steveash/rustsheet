---
id: T-3064
title: "Generator M2 criteria batch with positive text-number, tilde and boolean-text matches"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m2 holds oracle-backed cells where each criterion T-2717 added to the pool has at least one positive match: a text-number (`"1"`, `"2.5"`, `"=2"`, `"1E1"`), a tilde escape and a boolean text (`"TRUE"`, `"=FALSE"`).

## Context
The reviewer-excel-fidelity pass on #2017 (T-2717), as re-run by the steward on 2026-10-28, found a gap in its two stored gen-m2 cases (7d61ca5cd4679d78, a073d3573617bf8e). Every cell with a text-number, tilde or boolean-text criterion returns 0 or `#VALUE!` (the *IFS ranges differ in shape). So the oracle never shows `"1"` matching a numeric 1, or `"=FALSE"` matching FALSE. The batch predates T-2719 (*IFS ranges share a shape). Only the `"<="&DATE(1900,1,1)` criterion has hits.

## Acceptance criteria
- [ ] A new seeded gen-m2 batch (or `--focus` on COUNTIF/SUMIFS/COUNTIFS) whose oracle cells include a nonzero match for each new criterion kind; the eval passes or failures are owned by tasks.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
