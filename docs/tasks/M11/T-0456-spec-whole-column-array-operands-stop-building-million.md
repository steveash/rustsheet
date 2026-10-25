---
id: T-0456
title: "Spec: whole-column array operands stop building million-row arrays"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ for evaluating element-wise operations over whole-column or whole-row
ranges (`$D:$D=K2`, `IF(D:D=x,F:F)`, `MAX(IF(...))`) without building a 1,048,576-row array per
operand. Results stay bit-identical. The spec is reviewed and splits the work into S/M tasks.

## Context
Profiling corpus-bench 59932 after T-0455: its 150 CSE formulas such as
`MIN(IF($D:$D=K2,$F:$F))` still take about 55 s, roughly 0.37 s each. Each one builds several
`Vec<Vec<ArrayNode>>` of 1,048,576 one-element rows: the range (`Model::evaluate_range`,
cast.rs), the comparison (`handle_comparison`, arithmetic.rs), and IF's result (`if_array`,
functions/logical). Nearly all rows lie past the sheet's used area and are `Empty`.

`SearchVector` (lookup_table.rs) already keeps an evaluated prefix plus a virtual empty tail for
lookups. The spec should decide whether arrays get a similar representation (a used prefix plus
one repeated tail row), how element-wise operators, IF, aggregates and spills consume it, and how
the 2^20-cell bound (docs/excel-semantics/arrays.md) and seen-empty records still apply.

## Acceptance criteria
- [x] docs/specs/SPEC-XXXX covers the representation, every consumer it must handle, the
      invariants that keep results identical, and the directive 5 limits.
- [x] Implementation tasks are filed, each S/M, with corpus-bench 59932 (55 s eval, 78 s through the
      harness) as the measure: under 60 s through the harness leaves its `!Timeout` baseline line.
- [x] Reviewers approve the spec.

## Out of scope
- The implementation itself (the follow-up tasks).

## Outcome
- docs/specs/SPEC-0020-whole-column-array-operands.md: an aggregate's argument that is an
  element-wise expression of whole columns (range and scalar leaves, operators, IF) is evaluated
  as a used head plus one repeated tail row, decided from the AST before evaluation; the aggregate
  walk visits the same elements in the same order through the shared per-element body, so results,
  error precedence, reads and restarts are unchanged. No general array representation change.
- Filed T-0469 (core, measured on corpus-bench 59932 and 17049), T-0470 (element-wise functions),
  T-0471 (SUMPRODUCT). Profiling 17049 (T-0430) showed the same cost as 59932.
- Reviews: security, quality APPROVE; architecture and Excel-fidelity APPROVE after the
  array-producing rule (no bare-range or scalar-condition-IF arguments), the owner-only tail skip,
  `array_of_errors` error tails and chain-walk recursion bounds were added.
