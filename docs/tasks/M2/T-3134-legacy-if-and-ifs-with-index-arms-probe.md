---
id: T-3134
title: "Legacy IF and IFS with INDEX arms: probe which shapes Excel shows without @ (IFS, 2-D range, computed row, mixed arms) and generalize returns_single_cell_index"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A legacy (pre-dynamic-array) formula whose root is an IF or IFS with INDEX value arms gets the implicit-intersection
`@` exactly when Excel's does. Its value and its FORMULATEXT both match, and the rule is backed by one oracle case
that covers the shapes T-2326 left UNVERIFIED.

## Context
- docs/excel-semantics/references.md, IF/IFS arm section (T-1935, T-1969, T-2326): `=IF(TRUE,INDEX(B1:B6,3))` shows
  no `@` in Excel (probes/legacy-if-arm-other-references M12). `=IF(TRUE,Rng)` and `=IFS(TRUE,B1:B6)` do. The engine
  drops the `@` only for the one evidenced shape. `returns_single_cell_index` and `is_single_cell_index`
  (crates/rustsheet-engine/src/expressions/parser/static_analysis/if_arm.rs) accept only an IF whose value arms are
  all `INDEX(one-column range, literal row >= 1)`. The entry says: "UNVERIFIED, so still `@`: IFS, a 2-D range
  (two-argument INDEX gives a whole row), a computed row (`INDEX(B1:B6,1+2)`), mixed arms."
- The guess matters for values, not only for the printed text:
  - a 2-D `INDEX(B1:C6,3)` is a whole row, so an `@` intersects it by column;
  - a computed row that can be 0 gives a whole column;
  - a mixed arm such as `IF(c,INDEX(B1:B6,3),5)` is one value either way.
  If Excel's rule is a static "can this arm be more than one cell" test, the probe must find where it draws the
  line. Candidates: (a) literal row/column arguments only; (b) any argument count that fixes one cell
  (`INDEX(B1:C6,3,1)`, a one-row range with one index); (c) an arm-kind test that ignores the arguments.
- Why opus: this is a static-analysis model fitted to evidence. The probe must separate (a), (b) and (c). Each
  shape needs a FORMULATEXT cell and a value cell placed off the range's rows or columns, so that a wrong `@`
  shows up as a wrong value. The change in static_analysis.rs must keep every cell of the three legacy-if-arm
  probes and probes/legacy-match-lookup-array-intersection passing.
- Method: the `excel-probe` skill, excel-graph, one case `probes/t3134-legacy-if-index-arm-shapes` (milestone M2,
  legacy cells as in probes/legacy-if-arm-other-references, with a Z-column control that proves the formulas
  stayed legacy). Shapes:
  - `IFS(TRUE,INDEX(B1:B6,3))`;
  - `IF(TRUE,INDEX(B1:C6,3))` and `IF(TRUE,INDEX(B1:C6,3,1))`;
  - `IF(TRUE,INDEX(B1:F1,3))` (one row);
  - `IF(TRUE,INDEX(B1:B6,1+2))` and `IF(TRUE,INDEX(B1:B6,A20))` with A20 = 3 and with A20 = 0;
  - `IF(TRUE,INDEX(B1:B6,0))`;
  - `IF(TRUE,INDEX(B1:B6,3),5)` and `IF(FALSE,INDEX(B1:B6,3),INDEX(C1:C6,2))`;
  - nested `IF(TRUE,IF(TRUE,INDEX(B1:B6,3)))`;
  - `CHOOSE(1,INDEX(B1:B6,3))`.
  Each shape is read at the root and under ABS.

## Acceptance criteria
- [x] `probes/t3134-legacy-if-index-arm-shapes` is committed with its excel-graph oracle sidecar. For every shape
      above it has a FORMULATEXT cell and at least one value cell off the range's rows (and off its columns, for
      2-D and one-row ranges).
- [x] Every cell of that case passes. probes/legacy-if-arm-range-argument, -b, legacy-if-arm-other-references and
      legacy-match-lookup-array-intersection keep all their passing cells (`make eval-check`: 0 regressions;
      `make eval-baseline` run).
- [x] `returns_single_cell_index` (or its replacement) states the fitted rule in its doc comment and cites the
      cells. A parser unit test covers one shape that gains `@` and one that loses it.
- [x] references.md's IF/IFS arm section replaces the "UNVERIFIED, so still `@`" sentence with the probed rule
      (VERIFIED, case id and cells). Shapes the probe did not cover are listed as UNVERIFIED.
- [x] If the probe shows cells no single rule fits within size S, the fitted cells pass, the rest are recorded as
      known failures by `make eval-baseline` (never hand-edited), and a follow-up task names them.
- [x] `make check` green; reviewers approve.

## Out of scope
- `@` insertion for functions other than IF, IFS and CHOOSE, and for OFFSET/INDIRECT arms (already probed,
  T-1969).
- Dynamic-array (cm) formulas, which get no `@`.
- The harness's legacy-parse adapter (T-1096).

## Outcome
probes/t3134-legacy-if-index-arm-shapes (167 cells, excel-graph) separates the candidates: Excel's line is (a),
literal arguments only, with whole numbers of 1 or more, and it ignores the range's shape (even the 2-D
`INDEX(B1:C6,3)`, `#REF!`). Literal arms beside the INDEX, nested IFs and CHOOSE drop the `@` too; IFS, a computed,
cell, text, non-integer or 0 index, and a range arm beside the INDEX keep it. `returns_single_cell_index` now
fits that rule and makes such an IF or CHOOSE a static scalar, so a call over it (ABS) gets no `@` either.
Eval delta: probes 0 regressions, the new case 167/167; the four neighbouring probes unchanged. No cell needed a
follow-up.
