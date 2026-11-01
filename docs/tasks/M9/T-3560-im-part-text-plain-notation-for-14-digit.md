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
- Evidence: probes-numeric/t3545-imsin-imcos-small-imag-sweep A128, A43, B49 and B81 (`make eval-show
  CASE=probes-numeric/t3545-imsin-imcos-small-imag-sweep`): digits equal, notation differs. The 15-digit
  `1.23456789012345E-06` is E notation (probes/eng-im-small-part, T-2056).
- Code: crates/rustsheet-engine/src/number_format.rs `value_to_text`, `TextRules::ComplexPart` (`MAX_PLAIN_LEN` 20,
  `MIN_COMPLEX_PLAIN_EXPONENT` -5).
- Semantics: docs/excel-semantics/engineering.md "An IM* part with 14 significant digits at exponent -6 is plain
  text (T-3545)". Whether the limit is 21 characters for every exponent, or exponent -6 only, is unprobed: probe
  13 digits at -7 and 14 at -7 first (`excel-probe` skill), then implement what the probe shows.
- More evidence (gap triage 2026-11-01, `make eval` on origin/main 8c8cfd396, excel-graph): 14 cells of T-3589's and
  T-3609's sweeps differ from Excel by notation only, digits equal, and no open task named them. 13 are 14 digits at
  exponent -6 (21 characters without the sign). One, t3589 S!C127, is 13 digits at exponent -7
  (`1.00578618257237-0.0000004257623550779i`, also 21 characters), so 13 digits at -7 is already answered: plain.
  Examples, Excel vs ours: t3589 S!A4 `IMSINH("6.09023237964955E-06")` `0.0000060902323796872` vs
  `6.0902323796872E-06`; S!B113 `IMCOSH("9.44469908419292E-06+0.99562846593553i")`
  `0.543975650586412+0.0000079250564654669i` vs `0.543975650586412+7.9250564654669E-06i`.
  - probes-numeric/t3589-im-hyperbolic-small-x-sweep.case.json (11): S!A4, A42, A61, B21, B99, B113, B131, B145, C127, D131, D149
  - probes-numeric/t3609-imsech-imcsch-small-real-sweep.case.json (3): S!D31, D175, D187

## Acceptance criteria
- [ ] A probe (excel-graph) settles the plain/E boundary for IM* parts at exponents -6 to -10 with 12 to 15 digits.
- [ ] The 4 cells above (A128, A43, B49, B81) and the 14 t3589/t3609 cells listed above pass; probes/eng-im-small-part and every other IM* text cell keep passing.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
