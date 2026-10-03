---
id: T-0118
title: "Importer: no panics on POI workbooks (!Panic) and resource limits for xmlbomb/external-link files"
milestone: M1
lane: engine
status: in-progress
priority: P0
size: M
depends_on: []
---

## Goal
Every `poi` case that is a baselined crash loads without panic: 47889, 52348, 60825, 65016, DataValidationListTooLong, bug60858, bug62181, invalid_cell_reference, link-external-workbook-a/b, xlsx-jdbc (`!Panic`); the `!EngineError` cases (54288, 54764*, 56278, 57914, 59021, 59746_NoRowNums, InlineString, LIBRE_OFFICE-128382-0, poc-shared-strings, style-alternate-content) either import or fail with a typed error; poc-xmlbomb* are rejected by a resource limit.

## Context
Found by T-0111's baseline update: these crash-only cases were recorded in `eval/baseline/poi.txt` as known failures (the T-0003 baseline predates T-0007's crash-only reporting). Workbooks are untrusted input (CLAUDE.md directive 5): panics and unbounded expansion are bugs.

## Acceptance criteria
- [ ] No `!Panic` line remains for `poi/*` in the baseline; panics replaced by errors.
- [ ] XML-bomb files are rejected with a typed limit error without large allocation.
- [ ] Baseline shrinks via `make eval-baseline`; `make check` green; reviewers approve.

## Out of scope
Making the cases evaluate correctly (see T-0112 for missing rels / Strict OOXML).
