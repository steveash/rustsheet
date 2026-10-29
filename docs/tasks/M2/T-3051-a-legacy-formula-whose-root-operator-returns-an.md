---
id: T-3051
title: "A legacy formula whose root operator returns an array constant's result shows its first element, not a spill"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A legacy (non-dynamic) formula whose root operator works on an array constant (`=0.1+{0.2;0.2}-0.3`,
`=1+{7;8}*2^-52-1`, `=0+{0.1,1}+{0.2,1}-({0.3,2})` stored as a plain `<f>`) shows the array's first element in its
own cell and spills nothing, as Excel does; the engine spills it (or is `#SPILL!` when the spill is blocked).

## Context
- Evidence (T-3047, probes/t3047-snap-array-operands sheet L, excel-graph 2026-10-28): legacy `=0.1+{0.2;0.2}-0.3` in
  G1 is 0 with G2 empty; legacy `=1+{7;8}*2^-52-1` in H2 is 0 (the first element; the second, in H2's own row
  position, would be 8 ULP = 1.7763568394002505e-15) with H3 empty; legacy `=0+{0.1,1}+{0.2,1}-({0.3,2})` in F1 is 0
  (ours `#SPILL!`, G1 blocks it). Known failures L!F1, L!G2, L!H3 in eval/baseline/probes.txt.
- The engine adds the implicit `@` of a legacy formula in `Model::legacy_formula_node`
  (crates/rustsheet-engine/src/legacy_formula.rs); docs/excel-semantics/references.md records that an array constant
  inside a call's operator takes its own `@` (`=N(@{2;3}+0)`, probes/legacy-array-at-shapes), but no probe covered an
  array constant under a **root** operator before T-3047. What FORMULATEXT shows for these cells (`=@{..}` on the
  constant or `=@(...)` on the whole) is unprobed: probe it first (excel-probe skill), then match it.

## Acceptance criteria
- [x] A probe case records FORMULATEXT and value of legacy root operators over array constants (row, column and 2-D
      constants, the constant left and right of the operator, nested under a unary minus), and the rule is in
      docs/excel-semantics/references.md.
- [x] The engine matches it: probes/t3047-snap-array-operands L!F1, L!G2 and L!H3 leave eval/baseline/probes.txt, with
      no new known failures.
- [x] `make check` green; reviewers approve.

## Out of scope
- The near-zero snap itself (T-3047); legacy CSE array formulas over a range (T-0950).

## Outcome
- Probe `probes/t3051-legacy-root-array-operators` (38 cells, excel-graph, 2026-10-28): in a legacy formula every
  array constant the root is made of through operators, parentheses and signs takes its own `@` and the cell shows
  its first element with no spill (`={2;3}` `=@{2;3}`, `=1+@{2,3}`, `=@{1,2;3,4}*2`, `=@{2;3}+@Z1:Z2`). Recorded in
  docs/excel-semantics/references.md next to the in-call operator rule.
- Engine: `add_root_legacy_intersection` puts the `@` on those constants (`add_operand_array_at`, shared with the
  scalar-argument rule) and the export drops it again (`remove_operand_array_at`). Unit test
  `legacy_root_array_constants_intersect_and_export_unchanged`.
- Tools: spec2xlsx wrote any cell formula ending in `}` without its last brace (XlsxWriter's `_prepare_formula`
  strips one trailing `}`), so Excel for the web refused the workbook (`FileCorruptTryRepair`); `_cell_formula` adds
  it back (test_spec2xlsx_trailing_brace.py).
- Eval: probes/t3047-snap-array-operands L!F1, L!G2, L!H3 leave eval/baseline/probes.txt; the new case passes 38/38.
