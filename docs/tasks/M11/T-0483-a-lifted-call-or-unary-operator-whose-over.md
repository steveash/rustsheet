---
id: T-0483
title: "A lifted call or unary operator whose over-bound array cannot fit is #SPILL! before the array-size cap's #ERROR!"
milestone: M11
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
A dynamic-array cell whose formula is a lifted call (`=DATE(11:1899,12,1)`), a unary operator or `%` over a
range (`=-H:J`), or another over-bound array that is not an operator broadcast is `#SPILL!` when the array's
shape cannot spill from the anchor, as Excel has it, instead of the array-size cap's `#ERROR!`, without building
the array (CLAUDE.md directive 5).

## Context
T-0480 made a root operator chain refuse such a spill from its operand shapes (`Model::evaluate_spilling_chain`,
`unspillable_broadcast` in crates/rustsheet-engine/src/arithmetic.rs). The other over-bound shapes still hit the
cap first: `=DATE(11:1899,12,1)` in C1 is `#SPILL!` in Excel (probes/lifted-range-rows C3, listed in
eval/suites/probes/deviations) and `#ERROR!` here (test_lifted_range_bound.rs
`whole_row_range_lifted_by_date_is_bounded`). docs/excel-semantics/arrays.md "Array results above 1,048,576
cells are #ERROR!" lists what the deviation still covers. Needs a design for where a lifted call's broadcast
shape is computed before the lift (SPEC-0011 driver, `lift_numbers`), only in a spill context.

## Acceptance criteria
- [ ] `=DATE(11:1899,12,1)` in C1 and `=-H:J` in C3 are `#SPILL!`; `ROWS(DATE(11:1899,12,1))` stays `#ERROR!`
      (decision 0014); probes/lifted-range-rows C3 leaves eval/suites/probes/deviations if it is listed there.
- [ ] No allocation above `limits::MAX_SIZE` on these paths.
- [ ] `make check` green; reviewers approve.

## Out of scope
- An over-bound array whose shape fits the grid from its anchor (Excel spills it): the decision-0014 deviation.
- Value contexts (`ROWS`, `SUM`, `INDEX`).
