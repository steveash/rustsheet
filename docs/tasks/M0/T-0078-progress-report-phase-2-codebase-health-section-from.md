---
id: T-0078
title: "Progress report phase 2: codebase health section from the quality ratchet files"
milestone: M0
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-0071]
spec: docs/specs/SPEC-0022-daily-progress-dashboard.md
---

## Goal
The dashboard has a generated `codebase` block with the quality ratchets and their trend, so the
hygiene line can leave docs/status.md.

## Context
- SPEC-0022 lists codebase metrics as a later phase.
- Inputs, read only:
  - quality/lint-debt.json, quality/duplication.json, quality/oversized-files.json and
    quality/evidence-floor.json;
  - the count of docs/decisions/0*.md;
  - the line count of docs/excel-semantics/*.md.
- History rows come from `progress_report/history.py` (T-0071). Add the new keys as optional, so
  older rows stay valid.

## Acceptance criteria
- [ ] Module `progress_report/codebase.py` writes the `codebase` block: lint-debt allow entries,
  duplicated lines, oversized files, decision records and excel-semantics lines, each with its
  delta since the last complete row.
- [ ] History rows gain an optional `codebase` object. Rows without it still load (test).
- [ ] Tests cover each count on fixture files.
- [ ] The regenerated page is committed. `make py-check` green; `make check` green; reviewers
  approve.

## Out of scope
- Line coverage, which needs a coverage tool.
- Removing the hygiene line from docs/status.md: the steward does that once this lands.

## Outcome
`progress_report/codebase.py` writes the generated `codebase` block (lint-debt allow entries,
duplicated lines, oversized files, functions below the evidence floor, decision records and
excel-semantics lines), each with its change against the last complete history row that has the
new optional `codebase` object (`n/a` until one exists, so the first run shows no deltas). History
rows without `codebase` still load (tested). `signed` moved from history.py to markdown.py so both
use one definition. The page was regenerated from a complete run (no eval delta; no harness or
engine behavior changed). The judgment blocks are from the previous run; the steward refreshes them.
