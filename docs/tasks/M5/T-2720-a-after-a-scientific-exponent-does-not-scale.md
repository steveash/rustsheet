---
id: T-2720
title: "A % after a scientific exponent does not scale: TEXT(2.675,'0.00E+00%') is 2.68E+00%"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A `%` placed after a scientific code's exponent digits is shown as a literal and does not multiply the value by 100,
as in Excel.

## Context
Found by T-2705. `TEXT(2.675,"0.00E+00%")` is "2.68E+00%" in Excel, while ours is "2.68E+02%" because it scales by 100
(probes/t2705-scientific-mantissa-rounding A13, excel-graph 2026-10-25). Unprobed: a `%` before the `E` (`0.00%E+00`), two
`%`, and a `%` in quotes. Probe those first (excel-probe skill). Code: crates/rustsheet-engine/src/formatter/format.rs (`scaled`
from `p.percent`) and the parser that counts `p.percent`.

## Acceptance criteria
- [ ] probes/t2705-scientific-mantissa-rounding A13 passes and leaves eval/baseline/probes.txt; the new probes pass.
- [ ] number-formats.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scientific mantissa rounding (T-2705).
