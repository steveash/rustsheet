---
id: T-0244
title: "Criteria text comparison ordering around CHAR(160) and spaces (probes/criteria-padded-text C13)"
milestone: M2
lane: functions-criteria
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
COUNTIF text comparison operators (`>`, `<`) order strings around CHAR(160) and spaces exactly as Excel does.

## Context
probes/criteria-padded-text C13 (`">"&CHAR(160)&"0"` over text cells "1","1 "," 1",CHAR(160)&"1","1"&CHAR(160)) is 4 in Excel, 1 in the engine, which compares raw lowercase strings in `result_is_greater_than_string` (functions/criteria.rs). Needs more probes to find Excel's collation, so tier opus. See docs/excel-semantics/criteria.md.

## Acceptance criteria
- [ ] probes/criteria-padded-text C13 passes and the baseline entry is removed; collation rule documented.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
