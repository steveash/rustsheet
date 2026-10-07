---
id: T-0279
title: "Coerce ( 1 ), ($1) and (1%) text to negative numbers"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Text such as `( 1 )`, `($1)` and `(1%)` coerces to -1, -1 and -0.01 as in Excel.

## Context
Oracle probe probes/paren-negative-variants (T-0277) shows Excel accepts padding, currency and percent inside the parentheses; the engine returns #VALUE! (cells B3,C3,B5,C5,B6,C6,E2,E3,E4). `(-1)`, `((1))` and `(1)x` stay #VALUE!. Code: `parse_formatted_number` in formatter/format.rs. See docs/excel-semantics/coercion.md "Parenthesised number text".

## Acceptance criteria
- [x] All cells of probes/paren-negative-variants pass; baseline shrinks via `make eval-baseline`.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`parse_formatted_number` now parses the text inside parentheses through the percent/currency/plain-number helpers (new `parse_percent_or_currency`, `parse_plain_number`), so `( 1 )`, `($1)`, `(1%)` are negative; `(-1)`, `((1))` stay errors. Eval: probes 9 fixed, 0 regressions (221 -> 212).
