---
id: T-2072
title: "Format a trailing comma after a fraction denominator as #VALUE!"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Fixed-denominator fraction formats with a `,` after the denominator (`# ?/4,`, `# ?/4,,`) match Excel.

## Context
Probe cells rows 5-6 of probes/fraction-format-features (docs/excel-semantics/number-formats.md, Fraction formats, T-2061): `# ?/4,` is #VALUE! except on 0 ("0    "); `# ?/4,,` prints the value's digits followed by zeros to ~300 digits and #VALUE! on 1234.5. We print "0    " / "-0    ". Code: formatter/fraction.rs.

## Acceptance criteria
- [ ] Every mismatching cell named above passes in probes/fraction-format-features, `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
