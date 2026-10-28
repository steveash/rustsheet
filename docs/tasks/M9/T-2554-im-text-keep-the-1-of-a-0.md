---
id: T-2554
title: "IM* text: keep the 1 of a -0.9999999999999999 part, plain digits for 1E+20 (sweep P43, D18, P18)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IM* text results print as Excel does in probes-numeric im-last-digit-sweep P43, D18 and P18.

## Context
Found by the T-2044 review (engineering.md "IM* last-digit sweep", "Text quirks" bullet):
`IMCOT("-57.9375+18.7089i")` is "-4.00011374673746E-17-1i" in Excel (imaginary part
-0.9999999999999999 keeps its "1"; an exact -1 is "-i", P44), ours "-4.00011374673746E-17-i";
`IMCSC("1E-20")` and `IMCOT("1E-20")` are "100000000000000000000", ours "1E+20". Likely the
`y == -1.0` test runs on the 15-digit-rounded part (rounding -0.9999999999999999 to -1), and
`complex_part_to_text`'s plain/E switch at 1E+20. Code: `impl fmt::Display for Complex` in
crates/rustsheet-engine/src/functions/engineering/complex/mod.rs, number_format.rs.

## Acceptance criteria
- [ ] P43, D18 and P18 of probes-numeric/im-last-digit-sweep print Excel's text; check other
      IM* probes for parts that round to ±1 or reach 1E+20 (no new known failures).
- [ ] engineering.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
