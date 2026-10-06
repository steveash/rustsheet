---
id: T-0143
title: "Truncate numeric literals and number text to 15 significant digits on parse"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Numeric literals, `"text"+0` and `VALUE("text")` keep only 15 significant digits, truncating
digits 16+ (not rounding), as Excel does.

## Context
docs/excel-semantics/numbers.md "15-digit literal parsing" (VERIFIED; probes-numeric/23733e52a2ff91ab
`literal-15-digit/*`, probes/precision-and-rounding A11). Found again by T-0107:
probes/number-to-text-notation A22 `=CONCAT(123456789012345678)` → "123456789012345000" and A29
`=CONCAT(12345.6789012345678)` → "12345.6789012345"; the engine parses the full literal, so the
15-digit text rounds instead ("…346000", "…2346"). Number→text itself is correct (T-0107).
Code paths: the formula lexer's number token and the text→number coercion (`cast.rs`).

## Acceptance criteria
- [x] probes/number-to-text-notation A22 and A29 and precision-and-rounding A11 pass; the
      probes-numeric `literal-15-digit/*` known failures shrink; no regressions.
- [x] One truncation helper is shared by the lexer and the text→number coercion.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added `parse_excel_number` (truncates to 15 significant digits), shared by the lexer, `cast_number` and `parse_formatted_number` (VALUE). probes known failures 135 -> 127 (8 fixed); probes-numeric (ungated) 2253 -> 2352 passing cells (all 300 literal cells and the VALUE cells); no regressions. Follow-up T-0150 for the remaining raw parse sites and an oracle case for `"text"+0`.
