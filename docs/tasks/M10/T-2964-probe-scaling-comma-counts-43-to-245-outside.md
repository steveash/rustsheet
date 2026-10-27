---
id: T-2964
title: "Probe scaling-comma counts 43 to 245 outside fractions"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT with 43 to 245 trailing scaling commas outside a fraction prints what Excel prints.

## Context
T-2842 found that Excel keeps the scaling-comma exponent -3n in a signed byte: `TEXT(5,"0"&REPT(",",252))` is
"5000000000000" (probes/t2842-dollar-run-contexts K1:K3, K10:K11), and `formatter/fraction.rs` `comma_exponent`
applies the wrap from 43 commas on (-129 wraps to 127). Counts 43 to 245 are unprobed: probe `"0"&REPT(",",n)` on 5 and
on 1E-100 for n = 42, 43, 44, 50, 85, 86, 100, 128, 171, 200, 245 (with `"0.0"` and `"#,##0"` variants), and fix
`comma_exponent` if Excel differs. number-formats.md "Many scaling commas wrap".

## Acceptance criteria
- [ ] A probe of those counts is oracle-filled and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
