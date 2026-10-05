---
id: T-0143
title: "Truncate numeric literals and number text to 15 significant digits on parse"
milestone: M1
lane: engine
status: in-progress
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
- [ ] probes/number-to-text-notation A22 and A29 and precision-and-rounding A11 pass; the
      probes-numeric `literal-15-digit/*` known failures shrink; no regressions.
- [ ] One truncation helper is shared by the lexer and the text→number coercion.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
