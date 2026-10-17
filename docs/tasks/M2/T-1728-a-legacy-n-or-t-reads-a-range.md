---
id: T-1728
title: "A legacy N or T reads a range argument as a reference, with no @"
milestone: M2
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy (non-array) `=N(Z1:Z2)` off the range's rows is 7 (Z1's value) with FORMULATEXT `=N(Z1:Z2)`, as in Excel, instead of `#VALUE!` from an added `@`.

## Context
probes/legacy-n-t-reference-args (excel-graph, 2026-10-16, T-1713), legacy row 10, FORMULATEXT row
20, Z1:Z2 = 7, 8: `=N(Z1:Z2)` 7 (Q), `=N(OFFSET(Z1:Z2,0,0))` 7 (C), `=N(IF(1,Z1:Z2))` 7 (J), all
shown with no `@`. N and T take a reference: a legacy formula does not intersect a range there, nor
a range in an IF/CHOOSE value arm under them, nor put `@` on OFFSET. N of a multi-cell range is its
top-left cell already (information.md "N over an array lifts element-wise",
probes/info-logic-argument-modes A6:A7). Ours adds the `@` (`legacy_intersects`,
expressions/parser/static_analysis/legacy.rs and signature.rs), so these are `#VALUE!` off the rows.
Keep the `@` on array-valued calls under N/T as it is (`N(@CHOOSE(1,{2;3}))` gives 2, where Excel's
`#VALUE!` needs T-1727's legacy marker; removing the `@` there would make the cell spill).
docs/excel-semantics/references.md "Legacy N and T over a reference-returning call".

## Acceptance criteria
- [ ] probes/legacy-n-t-reference-args S!C10, C20, J10, J20, Q10, Q20 leave eval/baseline/probes.txt; no regressions (full eval, ironcalc-calc and poi too: the xlsx import uses the same rewrite).
- [ ] references.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- N/T over array-valued calls (T-1713, T-1727).
