---
id: T-2598
title: "Probe engineering scientific carry and half-way mantissas (##0.0E+0 on 999.96, ##0E+0 on 0.0145)"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT on engineering scientific codes matches Excel when a rounding carry crosses a power of ten and when the mantissa sits on a decimal half.

## Context
Follow-up of T-2226. `decimal_round::round_engineering` rounds an engineering mantissa (two or more integer placeholders)
after the exponent shift. Two parts of it have no oracle evidence (number-formats.md, the T-2226 entry):
- Carry: ours moves to the next exponent multiple (`##0.0E+0` on 999.96 is "1.0E+3").
- Half-way: it rounds the binary double with `to_precision`, not the 15-digit decimal half away from zero that
  `round_shown_place` uses for non-scientific codes (T-2206). So `##0E+0` on 0.0145 may differ from Excel.
Probe TEXT with `##0.0E+0` on 999.96, 99.96, 9999.96 and -999.96; `00.0E+0` on 9.96 and 99.96; `##0E+0` on 0.0145, 0.0125
and 1.5E-5; `##0.00E+00` on 999.995. Add a grouped code (`#,##0.0E+0` on 1/3, 12345.6) for the group size, which is also
unprobed. Fill through excel-graph.

## Acceptance criteria
- [ ] A probe case with the cells above is filled through excel-graph, and number-formats.md states the rules it shows.
- [ ] Ours matches every probe cell, or each mismatch has its own follow-up task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
