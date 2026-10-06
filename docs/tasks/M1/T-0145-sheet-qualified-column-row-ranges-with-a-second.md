---
id: T-0145
title: "Sheet-qualified column/row ranges with a second prefix give #NAME? instead of #ERROR! (Data!A:Data!B)"
milestone: M1
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`Data!A:Data!B`, `Data!A1:Data!B`, `Data!A:Data!B1`, `Data!A:Data!A`, `Data!B:Data!A` and `Data!A1:B` evaluate to `#NAME?` as in Excel, not `#ERROR!`.

## Context
Found by T-0139. Excel reads the bare column or cell after the colon as a (sheet-scoped) name, so the formula parses and gives `#NAME?`; the engine's lexer fails (`#ERROR!`). Failing cells: probes/sheet-qualified-both-corners Calc!A1, A5, A6, A7, A11, A12. Rule: docs/excel-semantics/references.md "Sheet-qualified column or row corners on both sides are not a range". Code: `expressions/lexer/ranges.rs`. `Data!1:Data!2`, `Data!A1:Data!2`, `Data!$A:Data!$B` cannot be stored in a workbook, so they have no oracle value.

## Acceptance criteria
- [ ] The six cells above pass; the baseline shrinks accordingly.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
