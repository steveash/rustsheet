---
id: T-0241
title: "Cap SUBSTITUTE per-element result length"
milestone: M2
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
SUBSTITUTE builds each element result without a length cap, so one cell can allocate far more than 32767 chars.

## Context
Found by the T-0232 security review: `substitute_element` in `crates/rustsheet-engine/src/functions/text/common.rs` runs the replacement before any budget check (e.g. a 32K string with every char replaced by a 32K new_text is ~1 GB). Needs Excel evidence for the 32767-char result limit (probably `#VALUE!`), else a DEVIATION entry (decision 0014).

## Acceptance criteria
- [ ] SUBSTITUTE results longer than 32767 chars return an error without building the full string; test covers it; documented in docs/excel-semantics/text.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
