---
id: T-0283
title: "Typed currency input gets $#,##0_);[Red]($#,##0) and typed percent 0%"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Text typed into a cell picks the number format Excel picks: `$1`, `($1)` and `($1,000)` get
`$#,##0_);[Red]($#,##0)` and `1%`, `(1%)` get `0%` (the engine gives `$#,##0` and `#,##0%`).

## Context
Evidence: probes/typed-paren-number-input (excel-graph 2026-10-07, T-0280); the formats are in the
oracle's `ENTERED` log and recorded in docs/excel-semantics/coercion.md ("Parenthesised number
text is negative"). Values already match. Code: `parse_percent_or_currency` in
crates/rustsheet-engine/src/formatter/format.rs. The decimal forms (`$1.50`, `1.5%`, `(1.5%)`),
suffix currency and the `-$1` form are unprobed: add them as `{"entered": ...}` cells to a probe
case first and run `rustsheet-oracle fill` (its stderr `ENTERED` lines carry the formats), then
match them too.

## Acceptance criteria
- [ ] A probe case with the decimal and sign forms as `entered` cells has an oracle sidecar; the formats are recorded in coercion.md.
- [ ] `set_user_input` gives Excel's format for every probed form (unit tests in test_set_user_input.rs).
- [ ] `make check` green; reviewers approve.

## Out of scope
Non-en-US locales.
