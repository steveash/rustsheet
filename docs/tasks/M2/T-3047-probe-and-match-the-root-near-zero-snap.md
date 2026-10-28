---
id: T-3047
title: "Probe and match the root near-zero snap on array operands of + and - (spilled and legacy-intersected formulas)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Excel evidence for whether the near-zero snap of a formula's outermost `+`/`-` applies element by element when an
operand is an array or a multi-cell range, and the engine matching it in both the dynamic (spilled) and the legacy
(implicitly intersected) form.

## Context
- docs/excel-semantics/numbers.md "Near-zero snapping": the outermost `+`/`-` of a cell formula snaps a result below
  8 ULP of the smaller addend to 0 (`=0.1+0.2-0.3` is 0), but its "Implemented in" line says "Array operands of `+`/`-`
  are not snapped (unprobed)". The code says the same: `Model::handle_outermost_sum` in
  crates/rustsheet-engine/src/arithmetic.rs snaps only when both operands are numbers ("Array operands are not snapped
  (no evidence)"), and `Model::evaluate_formula_root` passes `snaps` into `evaluate_spilling_chain` for the spilled path.
- No probe case has an array or range operand at a snapping root: probes/near-zero-snap-groups,
  near-zero-sum-forms, near-zero-aggregate-forms, near-zero-snap-name-bodies and precision-and-rounding hold scalar roots
  only, and probes-numeric `snap-addsub/*` is scalar too.
- Wrong either way is visibly off (5.551115123125783e-17 against 0), so this is a semantic gap, not last-digit work.
  The boundary (k = 7 snaps, k = 8 does not, for addends near 1: T-0110, T-2714) must be re-checked per element,
  which is why this is opus work (numeric boundary, unclear evidence).

## Acceptance criteria
- [ ] A new excel-graph probe case `probes/t3047-snap-array-operands` (milestone M2, decision 0043) reads back, as
      dynamic-array formulas: `={0.1,1}+{0.2,1}-{0.3,2}`, `=A1:A3+B1:B3-C1:C3` over 0.1/0.2/0.3 rows mixed with rows
      that do not cancel, `={0.1}+0.2-0.3` (a one-element array), `=0.1+{0.2;0.2}-0.3` (scalar operands broadcast),
      and a per-element boundary pair (`=1+{k1;k2}*2^-52-1` style rows with k = 7 and k = 8 on both the `+` and the
      `-` side, as numbers.md's scalar rule reads them).
- [ ] The same case stores legacy (non-dynamic) forms that implicitly intersect a range operand on a row inside it
      (`=A1:A3+B1:B3-C1:C3` typed into row 1 as a legacy formula) and an `INDEX(...)` of a spilled snapping root.
- [ ] numbers.md "Near-zero snapping" records the rule with the cell ids and drops the "(unprobed)" note; the
      "Implemented in" line and the `handle_outermost_sum` doc comment cite the evidence.
- [ ] The engine matches every cell of the new case (`make eval-show CASE=probes/t3047-snap-array-operands` shows no
      failing cell), with unit tests in the engine's arithmetic tests for each probed shape; probes known failures do
      not rise.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Snap under value pickers (IF, LET, CHOOSE, ...): T-3049.
- Snap inside SUMIF, SUMIFS, AVERAGE or SUMPRODUCT: T-3048.
- Any change to the scalar snap rule or its 8-ULP bound.
