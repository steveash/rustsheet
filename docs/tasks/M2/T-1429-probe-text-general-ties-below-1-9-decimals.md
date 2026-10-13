---
id: T-1429
title: "Probe TEXT General ties below 1 (9 decimals) now rounded half away on the 15-digit decimal (follow-up of T-1422)"
milestone: M2
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(x,"General")` for ties below 1 (for example `0.0000000015`, `0.1234567895`) is settled by oracle evidence.

## Context
T-1422 made `format_general` round half away from zero on the 15-digit decimal at every magnitude, but probes/text-general-ties-wide-fixed only covers ties of 1 or more. Code: `format_general` (formatter/general.rs).

## Acceptance criteria
- [ ] a probe case covers sub-1 ties; `format_general` and docs/excel-semantics/number-formats.md match the oracle.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/text-general-ties-below-one` (excel-graph, 19 cells) covers sub-1 ties at the 9-decimal limit, near-ties, negatives and E-notation ties. The engine already matched every cell, so no code change. Recorded in docs/excel-semantics/number-formats.md. Eval delta: one generated C line added to eval/baseline/probes.txt, no F lines.
