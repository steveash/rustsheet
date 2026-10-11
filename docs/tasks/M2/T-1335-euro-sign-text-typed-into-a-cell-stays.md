---
id: T-1335
title: "Euro-sign text typed into a cell stays text in en-US (1€, €1, -1€ are General text)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed `1€`, `1.5€`, `-1€` and `€1` stay text with General format in en-US, as in Excel; the engine reads them as numbers.

## Context
Evidence: probes/typed-currency-percent-input B23:C26 (excel-graph 2026-10-11), coercion.md "Typed currency and percent formats". Check how the model's currencies list is built (model.rs set_user_input, `parse_formatted_number` callers) and whether the euro tests in test_set_user_input.rs (test_currencies_eur_*) encode IronCalc behavior that Excel en-US does not share; VALUE/coercion of `"1€"` is a separate, already probed path (paren-number-text-euro-suffix).

## Acceptance criteria
- [ ] B23:C26 of probes/typed-currency-percent-input pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
