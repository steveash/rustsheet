---
id: T-0003
title: "Add Apache POI Excel-authored workbooks as suite `poi`"
milestone: M0
lane: harness
status: done
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

## Outcome
Suite `poi` (pinned apache/poi `9958a2a`, `test-data/spreadsheet`, fuzzer corpus excluded): 334 cases, ~3.5k checked cells, ~82% bit-exact; known failures baselined (two near-timeout cases excluded until T-0111). Only Excel-authored caches are scored, so far fewer than the ~31k formula cells in the raw corpus count. Follow-ups: T-0111 (two 60s timeouts), T-0112 (import failures).
