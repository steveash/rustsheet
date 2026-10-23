---
id: T-2455
title: "TRANSPOSE of a single value returns that value (not a 1x1 array)"
milestone: M7
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
TRANSPOSE of a single value (scalar or one cell) returns that value, not a 1x1 array, so a lifting function over it does not lift.

## Context
Evidence: probes/t2414-gen-m7-unexplained A291, A301 and docs/excel-semantics/text.md "TRANSPOSE of a single value is that value". `COLUMNS(TEXTSPLIT(TRANSPOSE("a,b"),","))` is 2 in Excel, 1 in ours. `ISREF(TRANSPOSE(cell))` is FALSE.

Owned gen-m7 cells (T-2414):
- f7e94a817e5385ac A2882 `=TEXTSPLIT(TRANSPOSE(SUM(Data!B6,Data!A9:A9)),Data!A3:C7)`: B2882 Excel "", ours <empty>

## Acceptance criteria
- [ ] probes/t2414-gen-m7-unexplained cells A291 and A301 pass; lines leave eval/baseline/probes.txt.
- [ ] Visible parity: gen-m7/f7e94a817e5385ac B2882 (Excel "", ours empty) passes and its line leaves eval/baseline/gen-m7.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Arguments the probe did not cover.

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type, error or value, so the task is in the campaign; done when they have Excel's.
