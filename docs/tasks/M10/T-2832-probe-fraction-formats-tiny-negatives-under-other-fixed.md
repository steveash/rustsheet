---
id: T-2832
title: "Probe fraction formats: tiny negatives under other fixed denominators and grouped integers past 15 digits"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's answer for the fraction-format cases T-2074 implemented from one oracle cell each is pinned by a committed probe.

## Context
T-2074 made a negative value with no integer part and a zero numerator print no minus (evidence: probes/fraction-format-features F9 `?/100` and F13 `?/00` on -0.001 only) and grouped the integer digits under `#,##0 ?/4` (evidence: I10, 1234.5 only). Unprobed: `?/4` on -0.1, `??/??` and `?/?` on tiny negatives, `#,##0 ?/4` on 1234567.5 and on an integer past 15 significant digits, `?/?` and `# ?/?` on -0.001 (variable denominators), and Indian grouping `#,##,##0 ?/4`. Code: formatter/fraction.rs `format_fraction`; rule in docs/excel-semantics/number-formats.md "Fraction formats".

## Acceptance criteria
- [ ] A probes case covers the cells above, oracle-filled (excel-graph); number-formats.md states the rules; any mismatch with ours is fixed or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
