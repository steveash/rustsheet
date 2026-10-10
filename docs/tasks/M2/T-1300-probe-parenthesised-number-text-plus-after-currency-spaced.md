---
id: T-1300
title: "Probe parenthesised number text: plus after currency, spaced trailing currency, euro variants"
milestone: M2
lane: harness
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for `0+"($+1)"`, `0+"(1 $)"`, `0+"(€-1)"` and `0+"(1€)"`, which the engine rejects by inference (T-0282).

## Context
T-0282 rejects a sign after a currency and a trailing currency inside parentheses for every currency symbol; only `$` forms were probed (probes/paren-number-text-forms). See coercion.md "Parenthesised number text is negative"; code: `parse_formatted_number` in formatter/format.rs. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe case added with oracle values for the four forms; coercion.md updated; engine adjusted if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
