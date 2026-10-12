---
id: T-0413
title: "xlsx export doubles quotes in every conditional-format text rule operator (port ironcalc 4d38ff0e)"
milestone: M11
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Exported conditional-format text rules (contains, does not contain, begins with, ends with) keep a value with a double quote intact on reload, as the equals rule already does.

## Context
Upstream IronCalc 4d38ff0e (2026-10-08) fixed `text_formula_str` in
`xlsx/src/export/conditional_formatting.rs`: only `TextOperator::Equals` doubled embedded quotes; the
other four operators wrote the value raw, so a value of two quote characters was written as four
and changed the rule after a round trip. Our copy, `crates/rustsheet-xlsx/src/export/conditional_formatting.rs`
`text_formula_str`, has the same bug (only `Equals` uses `escaped`). Port it with
`Ported-From: ironcalc@4d38ff0e495bb074fc4a95fc364f8ab5b6b5e600`, including its test.

## Acceptance criteria
- [ ] All five text operators use the quote-doubled value; a round-trip test covers each.
- [ ] The commit carries the Ported-From trailer.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Evaluating conditional formats.
