---
id: T-3711
title: "A lone 0 placeholder after a fraction prints its zero"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(12.5,"# ?/?|0 ""12""|\5")` prints "12 1/2|0 12|5" as Excel does: a `0` placeholder after a fraction prints its zero.

## Context
Found by T-3532: probes/t3532-numeral-digit-fields row 14, all columns (A14 "١٢ ١/٢|٠ 12|5"; ours "١٢ ١/٢| 12|5" drops the zero, also with ASCII digits). Probe the neighbouring forms (`# ?/? 0`, `# ?/? #`, `# ?/? 00`, a value whose fraction is zero) before generalizing. Semantics: number-formats.md "Numeral-system digits" (fields bullet) and "Fraction formats". Code: formatter/fraction.rs `format_fraction`.

## Acceptance criteria
- [ ] probes/t3532-numeral-digit-fields row 14 passes (S!A14:G14); a probe covers the neighbouring forms.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
