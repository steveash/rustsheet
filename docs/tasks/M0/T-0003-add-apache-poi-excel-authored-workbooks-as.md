---
id: T-0003
title: "Add Apache POI Excel-authored workbooks as suite `poi`"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: S
depends_on: [T-0002]
---

## Goal
Score RustSheet on Apache POI's spreadsheet test data (Apache-2.0) using Excel's cached values.

## Context
About 31k formula cells in the xlsx/xlsm subset (research report §A.1.2). Use the fetcher (pinned commit) or vendor only the Excel-authored xlsx files if total size < 20 MB.

## Acceptance criteria
- [ ] Suite `poi` exists, `require_excel_authored = true`, `baseline = "cells"`, `gate = true`.
- [ ] `make eval` reports it; panics/timeouts (if any) filed as engine P0 tasks with case ids.
- [ ] Baseline recorded via `make eval-baseline`; status.md scoreboard updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
.xls (BIFF) files.
