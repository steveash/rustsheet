---
id: T-1335
title: "Euro-sign text typed into a cell stays text in en-US (1€, €1, -1€ are General text)"
milestone: M2
lane: engine
status: done
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

## Outcome
`set_user_input` (model.rs) lists only `$` and the locale currency, so typed `1€`, `1.5€`, `-1€`, `€1` stay General text; cast.rs (VALUE/coercion) is unchanged. The IronCalc tests `test_currencies_eur_prefix`, `test_currencies_eur_suffix` and `test_sum_function_eur` encoded typed euro text as a number (not Excel en-US) and were replaced by `test_typed_euro_text_stays_text`; `test_sum_function` covers the `$` case. probes baseline 2388 -> 2380 (B23:B26, C23:C26 fixed); 3195 engine tests pass.
