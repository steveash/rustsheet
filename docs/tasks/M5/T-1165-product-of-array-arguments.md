---
id: T-1165
title: "PRODUCT of array arguments (#N/IMPL! today)"
milestone: M5
lane: functions-math
status: todo
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
- [ ] Copies!B1:B35 and the cells derived from them pass; no regression.
- [ ] Array elements that are text, booleans or errors behave as Excel shows (evidence cited).
- [ ] `make check` green; reviewers approve.

## Out of scope
- PRODUCT overflow and subnormal rules (already covered).
