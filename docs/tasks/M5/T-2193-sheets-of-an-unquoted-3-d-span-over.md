---
id: T-2193
title: "SHEETS of an unquoted 3-D span over cell-like sheet names (S1:S3!A1) gives #VALUE!"
milestone: M5
lane: functions-logic
status: in-progress
priority: P3
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
An unquoted 3-D span over sheets named like cells (`S1:S3!A1`) evaluates as Excel for the web does: `#VALUE!` for `S1:S3!A1` and 1 for `S3:S1!A1`, for SHEETS (SUM already matches).

## Context
From T-2181. probes/sheet-names-like-cells-3d-spans S1!B3 (`=SHEETS(S1:S3!A1)`, expected `#VALUE!`) stays a known failure in eval/baseline/probes.txt (the other cells pass). Rule and evidence: docs/excel-semantics/information.md "Probe trap: sheets named like cells". SUM(S1:S3!A1) (B1) already gives `#VALUE!`, so look at how SHEETS reads the unquoted span before its argument becomes a reference.

## Acceptance criteria
- [ ] probes/sheet-names-like-cells-3d-spans S1!B3 passes and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-21): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Out of scope
-
