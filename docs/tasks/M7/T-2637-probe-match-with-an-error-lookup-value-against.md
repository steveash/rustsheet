---
id: T-2637
title: "Probe MATCH with an error lookup_value against a lifted bad match_type element or a bad lookup_array"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence settles MATCH's per-slot error order when a scalar error lookup_value meets an array
match_type whose element is bad, and when the lookup_array is an error or 2-D.

## Context
T-1926 (#1121) lets a scalar error lookup_value lift over an array match_type; inside each slot the
lookup_value error wins first (`fn_match`, `functions/lookup_and_reference/mod.rs`). These are unprobed:
`MATCH(NA(),A1:A3,{0;"x"})`, `MATCH(NA(),A1:A3,{0;#DIV/0!})` (we give `{#N/A;#N/A}`),
`MATCH(NA(),#REF!,{0;1})` and `MATCH(NA(),A1:B3,{0;1})` with a 2-D lookup_array (we give a 2x1 `#N/A`).
The closest evidence is `probes/match-lookup-value-lift` E17 (array lookup_value, scalar match_type).
Also: docs/excel-semantics/lookup.md's note "(the second slot's text match_type, not the error,
decides; ...)" on the `MATCH(25,A1:A3,{0;"x"})` example reads as a rule for an error lookup_value; reword
it once the probe answers. Found by the 2026-10-25 steward spot-audit (fidelity and architecture
reviewers on PR #1121).

## Acceptance criteria
- [ ] An oracle-filled probe covers the four formulas above (values and spill shape, via ROWS and
  ERROR.TYPE wrappers where needed).
- [ ] lookup.md records the rule and the note is reworded; any mismatch gets a functions-lookup fix task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine.
