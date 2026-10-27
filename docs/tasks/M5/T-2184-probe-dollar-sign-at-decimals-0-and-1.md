---
id: T-2184
title: "Probe DOLLAR sign at decimals 0 and 1 for a negative value that rounds to zero"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether DOLLAR prints parentheses for a negative value that rounds to zero at `decimals` 0 and 1.

## Context
T-2148 drops the parentheses only for negative `decimals` (`DOLLAR(-0.5,-1)` is "$0", evidenced) and keeps them at 2 (`DOLLAR(-0.004)` is "($0.00)", evidenced). `decimals` 0 and 1 are unprobed, so the `decimals < 0` line in `fn_dollar` (functions/text/string_format.rs) is inferred. Rule: docs/excel-semantics/text.md "FIXED, DOLLAR and BAHTTEXT cut to 15 significant digits first".

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with `DOLLAR(-0.4,0)`, `DOLLAR(-0.04,1)`, `DOLLAR(-0.4,1)` and `DOLLAR(-0.5,0)`, oracle-filled; text.md updated; engine fixed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
