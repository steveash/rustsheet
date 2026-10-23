---
id: T-2556
title: "LINEST and LOGEST: a text const or stats is #VALUE!"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
LINEST and LOGEST are `#VALUE!` when const or stats is text, as in Excel.

## Context
Evidence: probes/t2455-transpose-single-value A7 `LINEST({1;2},{1;3},"x")` and A26 `LINEST({1;2},{1;3},TRUE,"x")` are `#VALUE!` in Excel; ours reads the text as a boolean and returns 0.5 (`eval_optional_bool` in functions/statistical/linest.rs). docs/excel-semantics/statistics.md "LINEST and LOGEST report error values first". Check whether `"TRUE"` text is accepted before deciding the rule (probe it with `excel-probe`), and whether TREND/GROWTH share the helper.

## Acceptance criteria
- [ ] probes/t2455-transpose-single-value A7, B7, A26, B26 pass; their lines leave eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TREND and GROWTH unless the probe covers them.
