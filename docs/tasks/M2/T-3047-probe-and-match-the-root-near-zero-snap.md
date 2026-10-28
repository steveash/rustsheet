---
id: T-3047
title: "Probe and match the root near-zero snap on array operands of + and - (spilled and legacy-intersected formulas)"
milestone: M2
lane: engine
status: done
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
- [x] A new excel-graph probe case `probes/t3047-snap-array-operands` (milestone M2, decision 0043) reads back, as
      dynamic-array formulas: `={0.1,1}+{0.2,1}-{0.3,2}`, `=A1:A3+B1:B3-C1:C3` over 0.1/0.2/0.3 rows mixed with rows
      that do not cancel, `={0.1}+0.2-0.3` (a one-element array), `=0.1+{0.2;0.2}-0.3` (scalar operands broadcast),
      and a per-element boundary pair (`=1+{k1;k2}*2^-52-1` style rows with k = 7 and k = 8 on both the `+` and the
      `-` side, as numbers.md's scalar rule reads them).
- [x] The same case stores legacy (non-dynamic) forms that implicitly intersect a range operand on a row inside it
      (`=A1:A3+B1:B3-C1:C3` typed into row 1 as a legacy formula) and an `INDEX(...)` of a spilled snapping root.
- [x] numbers.md "Near-zero snapping" records the rule with the cell ids and drops the "(unprobed)" note; the
      "Implemented in" line and the `handle_outermost_sum` doc comment cite the evidence.
- [ ] The engine matches every cell of the new case (`make eval-show CASE=probes/t3047-snap-array-operands` shows no
      failing cell), with unit tests in the engine's arithmetic tests for each probed shape; probes known failures do
      not rise. **Partly met:** every snap cell matches; the 3 legacy array-constant cells (L!F1, L!G2, L!H3) are
      new-coverage known failures owned by T-3051 (not snap behavior); no existing known failure rose.
- [x] `make check` green; reviewers approve.

## Out of scope
- Snap under value pickers (IF, LET, CHOOSE, ...): T-3049.
- Snap inside SUMIF, SUMIFS, AVERAGE or SUMPRODUCT: T-3048.
- Any change to the scalar snap rule or its 8-ULP bound.

## Outcome (2026-10-28)
- Probe probes/t3047-snap-array-operands (70 cells, excel-graph): Excel snaps the root `+`/`-` **element by element** with
  the scalar 8-ULP rule (`=1+{3;7;8}*2^-52-1` is 0, 0, 8 ULP; `=1-{3;4;7}*2^-52-1` is 0, -4 ULP, -7 ULP), over ranges,
  array constants, broadcast scalars and 2-D constants; readers of the spill (`=E3`, `INDEX(E3#,1)`, `SUM(E3:E6)`) see
  the snapped values, `INDEX(A1:A4+B1:B4-C1:C4,1)` is not snapped, and a legacy formula's intersected rows snap.
  Recorded in docs/excel-semantics/numbers.md.
- `Model::handle_outermost_sum` now snaps through the per-element operator, so scalars and array elements share one
  path; unit tests `array_operands_snap_element_by_element`, `legacy_intersected_ranges_snap_per_row`.
- Eval delta: probes +1 case (70 cells, 67 pass); 3 new-coverage known failures L!F1, L!G2, L!H3 are not snap cells:
  a legacy root operator over an array constant shows its first element in Excel and spills in ours. They are owned
  by follow-up **T-3051** (`make exit-owners MILESTONE=M2`: 0 unowned). No other suite changed (0 fixed, 0 regressions).
- Follow-ups: T-3051 (legacy root over an array constant), T-3052 (spec2xlsx drops a formula's trailing `}`, which
  corrupted the first fill; the case writes `-({0.3,2})` to avoid it). Also lands planner tasks T-3048 and T-3049.
