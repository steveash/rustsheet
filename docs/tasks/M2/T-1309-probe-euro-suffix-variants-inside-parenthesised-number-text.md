---
id: T-1309
title: "Probe euro suffix variants inside parenthesised number text"
milestone: M2
lane: harness
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for `0+"(1 €)"`, `0+"(1,000.5€)"` and `0+"(1e3€)"`, with unparenthesised `1 €` and `1€` as controls.

## Context
T-1300 found `(1€)` = -1; the engine now accepts any trailing euro form by inference. See coercion.md "Parenthesised number text is negative"; code: `parse_formatted_number` in formatter/format.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [x] Probe case added with oracle values; coercion.md updated; engine adjusted if Excel differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes/paren-number-text-euro-suffix (excel-graph): every parenthesised euro-suffix form is negative and the unparenthesised controls are positive; the engine already matched all 7 cells, so no engine change. coercion.md updated.
