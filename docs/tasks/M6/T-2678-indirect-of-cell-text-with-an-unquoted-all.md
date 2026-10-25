---
id: T-2678
title: "INDIRECT of cell text with an unquoted all-digit sheet name (INDIRECT(\"2024!B1\")) is #REF!, Excel reads the cell"
milestone: M6
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
`INDIRECT("2024!B1")` reads cell B1 of the sheet named `2024`, as Excel does.

## Context
Found in T-2660. probes/indirect-sheet-scoped-name Summary!D3 (excel-graph, 2026-10-25):
`INDIRECT("2024!B1")` is `"y2024"` in Excel, `#REF!` in ours. The quoted `"'2024'!B1"` form and the
name form `"2024!tn"` (D1, D2) already pass. The cell text goes through
`ParsedReference::parse_reference_formula` in `fn_indirect`. Reuse `is_indirect_unquoted_sheet` (expressions/utils) for the prefix rule
(crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs), whose lexer does not take an
unquoted sheet name starting with a digit. Excel's rule for INDIRECT prefixes is in
docs/excel-semantics/lookup.md "INDIRECT of `Sheet!Name` text": unquoted, a prefix may hold only
letters, digits, `_` and `.`; `"Co A!B1"`, `"P&L!B1"`, `"O'B!B1"` stay `#REF!`.

## Acceptance criteria
- [ ] probes/indirect-sheet-scoped-name Summary!D3 passes; no other cell of the case regresses.
- [ ] Formula parsing of a typed `=2024!B1` is unchanged (no Excel evidence yet that a formula
      accepts it; this task is INDIRECT text only).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed formulas with unquoted digit-leading sheet names.

## Outcome
`INDIRECT` text with an unquoted digit-leading sheet prefix is quoted before parsing
(`quote_digit_leading_sheet`); typed formulas are unchanged. probes Summary!D3 now passes
(probes known failures 2718 -> 2717, no regressions).
