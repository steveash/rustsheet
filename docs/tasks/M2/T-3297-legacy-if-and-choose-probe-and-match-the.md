---
id: T-3297
title: "Legacy IF and CHOOSE: probe and match the @ display of the INDEX-arm and CHOOSE-array shapes T-3134 and T-3168 left unprobed (named or table INDEX, negative or out-of-range index, bare range condition)"
milestone: M2
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
For a legacy (no `cm`) IF or CHOOSE, where Excel shows the implicit-intersection `@` and where it shows none is
decided by excel-graph evidence for every shape references.md still lists as UNVERIFIED. The engine's
`returns_single_cell_index` and `leaves_choose_index_whole` then match Excel on each of them.

## Context
- Gap triage 2026-10-30e. Every visibly-off M0-M2 cell has an open owner and gen-m2 is at 100%. The open M2 risk is
  in rules the engine extends by inference. references.md "Legacy IF passes a range arm whole (T-1935)" (the paragraph that ends
  "UNVERIFIED: an INDEX of a defined name or table area ...") lists these engine choices with no Excel cell:
  - an INDEX of a defined name or a table area as the arm (`IF(TRUE,INDEX(Rng,3))` with `Rng` = B1:B6,
    `IF(TRUE,INDEX(T[Col],3))`). The engine keeps the `@`;
  - a one-argument INDEX (`IF(TRUE,INDEX(B3))`) and a four-argument INDEX (`IF(TRUE,INDEX((B1:B6,C1:C6),3,1,2))`).
    The engine keeps the `@`;
  - a negated literal index (`INDEX(B1:B6,-3)`). The engine keeps the `@`;
  - a bare range condition (`IF(B1:B6,INDEX(B1:B6,3))`) and an out-of-range literal index (`INDEX(B1:B6,99)`).
    The engine drops the `@`;
  - (T-3168) an array-constant CHOOSE index over plain-cell, computed-scalar or text/boolean arms
    (`CHOOSE({1,2},A1,B1)`, `CHOOSE({1,2},A1+1,5)`, `CHOOSE({1,2},"a",TRUE)`). The engine moves the `@` to the call,
    as for I54. Over mixed range and scalar arms (`CHOOSE({1,2},H1:H2,5)`) the engine keeps `@{1,2}`.
- Evidence layout to reuse: probes/t3134-legacy-if-index-arm-shapes (295 cells, excel-graph, 2026-10-29). Each shape
  goes on one row as a legacy formula, under ABS and under SUM. Each cell has its FORMULATEXT beside it (columns
  H-M), so the probe scores the display and the value together. probes/legacy-choose-array-index (T-1882) is the
  CHOOSE-array precedent.
- Code: `returns_single_cell_index` (crates/rustsheet-engine/src/expressions/parser/static_analysis/if_arm.rs) and
  `leaves_choose_index_whole` (same file). Extend them; do not add a second classifier.
- Why opus: the rule may not be a simple extension. A defined name or table area may count as a "range" for the
  single-cell test or may not, and an out-of-range literal is a `#REF!` that Excel may still treat as one cell.
  The implementation follows whatever Excel shows, so the probe comes first in this PR.

## Acceptance criteria
- [ ] A new probes case `probes/t3297-legacy-if-index-arm-more-shapes` with an excel-graph oracle sidecar. It has one
      row per shape above, with value and FORMULATEXT columns at the root, under ABS and under SUM, as in t3134. At least
      100 scored cells.
- [ ] Every cell of the new case passes, and every cell of probes/t3134-legacy-if-index-arm-shapes and
      probes/legacy-choose-array-index still passes. `make eval-check`: 0 regressions; `make eval-baseline` run.
- [ ] references.md states Excel's rule for each shape with cell citations and drops the UNVERIFIED items the probe
      settles. Any shape Excel refuses to load is recorded as such.
- [ ] Unit tests in the static-analysis tests pin each new shape's display.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Dynamic-array (`cm`) formulas and typed entry (no `@` question there).
- IFS: every IFS takes the `@` (verified by T-3134).
