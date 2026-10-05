---
id: T-0242
title: "Probe padded numeric criteria against text cells and non-ASCII spaces"
milestone: M2
lane: functions-criteria
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with oracle evidence how COUNTIF criteria with a space-padded numeric operand (`"1 "`, `" 1"`, `"= 1"`, `"<> 1"`, `"> 1"`) match *text* cells ("1", "1 ", " 1"), and whether non-ASCII spaces (CHAR(160)) are ignored around the operand; make the engine match.

## Context
T-0238 (probes/criteria-padded-operand) probed numeric cells only. `parse_criterion_number` in `crates/rustsheet-engine/src/functions/criteria.rs` now trims ASCII spaces, so criterion `"1 "` is numeric; `result_is_equal_to_number` parses text cells without trimming, so text "1 " no longer matches it and text "1" now does. Neither is evidenced. Record findings in docs/excel-semantics/criteria.md.

## Acceptance criteria
- [ ] A probe case covers the text-cell and CHAR(160) forms above; engine matches the oracle values.
- [ ] criteria.md records the findings.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
