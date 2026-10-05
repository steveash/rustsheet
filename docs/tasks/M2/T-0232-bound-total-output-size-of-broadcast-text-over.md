---
id: T-0232
title: "Bound total output size of broadcast_text over large ranges"
milestone: M2
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`broadcast_text` (text functions LEFT, REPT, SUBSTITUTE, CONCATENATE, ...) has a bounded total output: a broadcast over a very large range cannot allocate tens of GB of text.

## Context
Found by the T-0226 security review. `broadcast_text` in `crates/rustsheet-engine/src/functions/text/common.rs` allocates rows x cols results with no cap on total bytes; per-element caps (REPT, CONCATENATE 32767 chars) still allow 1M cells x 32 KB. Needs a DEVIATION entry if Excel evidence does not cover it (decision 0014).

## Acceptance criteria
- [ ] A broadcast whose total result text exceeds a documented bound returns an error rather than allocating; test with a long scalar over a 1M-row range.
- [ ] DEVIATION entry in docs/excel-semantics if the bound differs from Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
