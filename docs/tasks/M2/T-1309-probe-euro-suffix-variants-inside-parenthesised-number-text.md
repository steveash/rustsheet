---
id: T-1309
title: "Probe euro suffix variants inside parenthesised number text"
milestone: M2
lane: harness
status: todo
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
- [ ] Probe case added with oracle values; coercion.md updated; engine adjusted if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
