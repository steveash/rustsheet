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
Probe cells rows 5-6 of probes/fraction-format-features (docs/excel-semantics/number-formats.md, Fraction formats, T-2061): `# ?/4,` is #VALUE! except on 0 ("0    "); `# ?/4,,` is value x 1E250 printed in full (253-255 chars) and #VALUE! on 1234.5 (over TEXT's 255-character limit). We print "0    " / "-0    ". Code: formatter/fraction.rs.

## Acceptance criteria
- [ ] Every mismatching cell named above passes in probes/fraction-format-features, `make eval-baseline` records the fix.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
