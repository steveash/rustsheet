---
id: T-1729
title: "T over an array lifts element-wise in a dynamic formula"
milestone: M2
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A dynamic `=T(IF(1,{"a";"b"}))` spills "a", "b", as in Excel, instead of one "".

## Context
probes/legacy-array-result S!G1:G2 (excel-graph, 2026-10-16, T-1713). T lifts over an array value
element-wise (expected: text kept, anything else ""; unprobed), the way N does since T-1616 (information.md "N over an
array lifts element-wise", the N arm in functions/information/mod.rs), while T of a multi-cell
range is still one "" with no spill (text.md "Text function argument modes", references.md "T of a
multi-cell range is not intersected", probes/text-argument-modes F9; keep that). Only this
one T-over-array cell is probed: add an error element and a number element (`T({"a",1,#N/A})`)
with excel-probe before generalizing. information.md "T over an array lifts element-wise".

## Acceptance criteria
- [ ] probes/legacy-array-result S!G1:G2 leave eval/baseline/probes.txt; probes/text-argument-modes F9 still passes; no regressions.
- [ ] information.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The legacy `=T(IF(...))` `#VALUE!` (T-1713).
