---
id: T-3560
title: "IM* part text: plain notation for 14-digit parts at exponent -6 (21 characters)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* functions write a 14-significant-digit part at decimal exponent -6 in plain notation, as Excel does
(`IMSIN("0-1.03615953980931E-06i")` is `-0.0000010361595398356i`), without changing any part that already matches.

## Context
- Evidence: probes-numeric/t3545-imsin-imcos-small-imag-sweep A128, A43, B49 and one more cell (`make eval-show
  CASE=probes-numeric/t3545-imsin-imcos-small-imag-sweep`): digits equal, notation differs. The 15-digit
  `1.23456789012345E-06` is E notation (probes/eng-im-small-part, T-2056).
- Code: crates/rustsheet-engine/src/number_format.rs `value_to_text`, `TextRules::ComplexPart` (`MAX_PLAIN_LEN` 20,
  `MIN_COMPLEX_PLAIN_EXPONENT` -5).
- Semantics: docs/excel-semantics/engineering.md "An IM* part with 14 significant digits at exponent -6 is plain
  text (T-3545)". Whether the limit is 21 characters for every exponent, or exponent -6 only, is unprobed: probe
  13 digits at -7 and 14 at -7 first (`excel-probe` skill), then implement what the probe shows.

## Acceptance criteria
- [ ] A probe (excel-graph) settles the plain/E boundary for IM* parts at exponents -6 to -10 with 12 to 15 digits.
- [ ] The 4 cells above pass; probes/eng-im-small-part and every other IM* text cell keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
