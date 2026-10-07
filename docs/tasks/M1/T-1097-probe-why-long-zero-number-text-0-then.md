---
id: T-1097
title: "Probe why long zero number text (0. then 305 zeros) is #VALUE!"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The rule that makes long zero number text #VALUE! is pinned by oracle evidence and recorded in docs/excel-semantics/numbers.md, so an engine task can implement it.

## Context
T-0154 found (probes/text-number-tiny-zero A15, A16; excel-graph 2026-10-07) that `"0."` followed by
305 or 320 zeros, with no exponent, is #VALUE! for `0+A` and `VALUE`, although the zero-mantissa rule
(exponent less fraction digits within ±308) would read it as 0. Whether this is a length limit on
number text (255? 300?), a fraction-digit limit, or something else is unknown. numbers.md
"Number text of 1E+308 or more is not a number" records the cells. Code: `parse_excel_number` in
crates/rustsheet-engine/src/number_format.rs, which reads both strings as 0 today.

## Acceptance criteria
- [ ] A probe case (excel-graph) brackets the cut-off: zero and nonzero number text of lengths around 255 and 300 (e.g. `0.` plus n zeros, `1.` plus n zeros, leading-zero integers, spaces), with LEN cells.
- [ ] numbers.md states the rule with the probe cells; an engine follow-up task is filed if the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
