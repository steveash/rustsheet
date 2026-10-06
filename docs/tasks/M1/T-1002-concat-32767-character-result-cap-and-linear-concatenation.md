---
id: T-1002
title: "CONCAT: 32767-character result cap and linear concatenation"
milestone: M1
lane: functions-text
status: todo
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
