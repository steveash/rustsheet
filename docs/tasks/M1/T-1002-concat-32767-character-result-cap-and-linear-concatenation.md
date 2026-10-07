---
id: T-1002
title: "CONCAT: 32767-character result cap and linear concatenation"
milestone: M1
lane: functions-text
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
CONCAT returns #VALUE! when the result exceeds 32767 characters and builds the string in linear time.

## Context
Review finding on T-0196. `fn_concat` (crates/rustsheet-engine/src/functions/text/common.rs) has no length cap and its top-level String arm rebuilds the string with `format!` per argument. 3-D spans let one formula join many more cells. A cap is a directive-5 limit: probe Excel's behavior (docs/excel-semantics) and add a DEVIATION entry only if the cap differs (decision 0014).

## Acceptance criteria
- [ ] Probe case with oracle values for a CONCAT result at and over 32767 characters.
- [ ] The cap matches the evidence; concatenation is linear.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other text functions.

## Outcome
Superseded by T-0243 (PR #203): the oracle (probes/text-result-limit C8, excel-graph) shows CONCAT past 32,767 characters is `#CALC!`, not `#VALUE!` as the Goal above assumed, and `fn_concat` now builds the text linearly (`ConcatText`). No DEVIATION needed. Boundary and UTF-16 evidence: T-0262.
