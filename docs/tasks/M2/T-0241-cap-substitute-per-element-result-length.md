---
id: T-0241
title: "Cap SUBSTITUTE per-element result length"
milestone: M2
lane: functions-text
status: done
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
- [x] SUBSTITUTE results longer than 32767 chars return an error without building the full string; test covers it; documented in docs/excel-semantics/text.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
SUBSTITUTE results over 32,767 chars are `#VALUE!` (excel-graph, probes/text-result-limit C1-C7), sized before building via `substituted_chars` (text/util.rs). New probe cells C8/C9 show CONCAT/TEXTJOIN give `#CALC!`: baselined as 2 new-coverage failures (probes 47 -> 49), follow-up T-0243. excel-desktop not used.
