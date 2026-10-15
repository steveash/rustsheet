---
id: T-1612
title: "Probe CONCATENATE with arguments and an error after the 32,767-unit cut"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that CONCATENATE arguments after the 32,767-unit cut change nothing and that an error argument after the cut still wins.

## Context
T-1580 reads the arguments after the cut for errors without appending them (`fn_concatenate`), covered only by the hand-written unit test `concatenation_keeps_a_lone_high_surrogate_at_the_limit` (A4, A5). Add the formulas `LEN(CONCATENATE(REPT("a",32766),"😀","zz",1))` and `CONCATENATE(REPT("a",32766),"😀",NA())` to a probe case with the excel-probe skill; record in coercion.md and fix ours if it differs.

## Acceptance criteria
- [x] A probe case with oracle values covers both; coercion.md updated and ours fixed if it differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
probes/concatenate-after-cut (9 cells, excel-graph, 2026-10-15): arguments after the cut change nothing and the first error wins; the engine matches all nine, so no fix. The lone-high-surrogate UNICODE cell is left to T-1581.
