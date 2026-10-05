---
id: T-0246
title: "Probe bare = criterion in SUMIF/AVERAGEIF/*IFS over empty-string and blank cells"
milestone: M2
lane: functions-criteria
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SUMIF, AVERAGEIF and the *IFS functions have oracle evidence for a bare `"="` criterion over an empty-string cell and a blank cell, confirming they behave like COUNTIF.

## Context
T-0227 changed `build_criteria_with` (Standard flavor) so `"="` matches blank cells only. Only COUNTIF is covered by probes (criteria-grammar C4, criteria-gaps C2); the other functions share the code path but have no oracle cells. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe case with SUMIF/AVERAGEIF/COUNTIFS/SUMIFS over a range holding an empty-string cell and a blank cell, with oracle values; any mismatch gets a fix or follow-up task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added `probes/criteria-ifs-bare-equals` (12 cells) with excel-graph oracle values; the engine already
matches all 12 (no engine change). Finding recorded in criteria.md. Eval delta: probes cases +1,
cells +12 passed, known failures unchanged.
