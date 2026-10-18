---
id: T-1557
title: "N of an array: probe and implement"
milestone: M5
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`N` of an array value gives what Excel gives instead of `#N/IMPL!`: `N(INDEX(D1:D5,{1}))` is 10 (probes/index-1x1-array-index H21).

## Context
Found by T-1507. `Model::fn_n` (functions/information/mod.rs) returns `#N/IMPL!` for `CalcResult::Array`. Only the 1x1 case is evidenced (H21, excel-graph). Probe (excel-probe skill) whether `N({10,20})`, `N({"a",2})`, `N({TRUE})`, `N({#N/A,1})` and `N(SEQUENCE(2))` take the top-left element or lift element-wise, then implement and record the rule in docs/excel-semantics/information.md.

## Acceptance criteria
- [ ] probes/index-1x1-array-index H21 passes; the new probe cells pass.
- [ ] The rule for multi-element arrays is recorded in information.md with its probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Already satisfied by T-1616 (N lifts element-wise; rule in docs/excel-semantics/information.md, probe
probes/n-array-lift). probes/index-1x1-array-index H21 is not in eval/baseline/probes.txt, so it passes. No engine change;
only the stale note in lookup.md was corrected. Eval delta: none.
