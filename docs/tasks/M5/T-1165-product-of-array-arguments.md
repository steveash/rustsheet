---
id: T-1165
title: "PRODUCT of array arguments (#N/IMPL! today)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PRODUCT multiplies the numbers of an array argument (`PRODUCT(IF(SEQUENCE(5),7))`,
`PRODUCT({2,3})`) as Excel does instead of returning `#N/IMPL!`.

## Context
- probes/geomean-single-and-copies Copies!B1:B35 (excel-graph, 2026-10-08, T-1123) hold Excel's
  PRODUCT of `IF(SEQUENCE(n),v)` arrays and of `0.5+MOD(SEQUENCE(20)*7,89)/89`; columns D-G derive from
  B, so 175 cells of that case fail with B. Excel's values equal the left-to-right double product.
- `fn_product` (crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs) sends
  a non-reference argument through `cast_aggregate_arg_to_number`, which has no array case. Reuse
  the aggregate iteration GEOMEAN uses (`for_each_value`) or the SUM array path; do not add a new
  walker. Check how text, booleans and errors inside an array argument count (probe them with the
  `excel-probe` skill if no case covers them).

## Acceptance criteria
- [x] Copies!B1:B35 and the cells derived from them pass; no regression.
- [x] Array elements that are text, booleans or errors behave as Excel shows (evidence cited).
- [x] `make check` green; reviewers approve.

## Out of scope
- PRODUCT overflow and subnormal rules (already covered).

## Outcome
`fn_product` now multiplies the numbers of an array argument (text/booleans skipped, errors propagate, as `for_each_value`). probes baseline: 159 known-failure lines removed (Copies!B1:B35 and derived cells), no regressions. 
Evidence: probes/product-array-elements (excel-graph) covers text, booleans, errors and all-non-numeric arrays; the engine matches. `fn_product` moved to product.rs verbatim apart from the array arm because mathematical.rs hit the 1000-line limit.
