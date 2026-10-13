---
id: T-1400
title: "Probe ROUND-family and CONCAT array edge cases (non-number elements, error past the text limit)"
milestone: M2
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for two array edge cases T-0222 implemented without a probe (excel-probe skill), with fixes if the engine differs.

## Context
Reviewer follow-ups on T-0222. (1) Non-number elements in the ROUND family: `ROUND({TRUE,"2.5",""},0)`, `ROUND({"abc"},0)` (`lift_numbers`/`number_at` cast like a scalar argument; unverified). (2) `CONCAT(REPT("a",32767),{"b",#N/A})` and an error mid-array past the limit (`fn_concat` array arm drops it like a range cell error; text.md marks it unprobed).

## Acceptance criteria
- [ ] A probe case with an excel-graph oracle covers both groups; the engine matches or the differences are fixed; docs/excel-semantics entries updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/round-concat-array-edges` (excel-graph) covers both groups; the engine already matched all 25 cells, so no code change. Findings recorded in docs/excel-semantics/arrays.md and text.md. Eval delta: probes +25 cells passing (one generated C line added to eval/baseline/probes.txt, no F lines).
