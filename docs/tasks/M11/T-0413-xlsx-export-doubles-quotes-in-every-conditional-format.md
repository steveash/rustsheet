---
id: T-0413
title: "xlsx export doubles quotes in every conditional-format text rule operator (port ironcalc 4d38ff0e)"
milestone: M11
lane: engine
status: done
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
- [x] All five text operators use the quote-doubled value; a unit test on `text_formula_str` covers each (a round trip cannot see it: the importer reads the `text` attribute, not the formula).
- [x] The commit carries the Ported-From trailer.
- [x] `make check` green; reviewers approve.

## Out of scope
- Evaluating conditional formats.

## Outcome
`text_formula_str` now writes the quote-doubled value for all five text operators. A unit test on
the function covers each operator; a save/load round trip cannot see the bug because the importer
reads the rule's `text` attribute, not the formula. Eval delta: none.

Ported-From: ironcalc@4d38ff0e495bb074fc4a95fc364f8ab5b6b5e600
