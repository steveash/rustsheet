---
id: T-2269
title: "SUBTOTAL 6 (PRODUCT) over no numbers is 0, not 1"
milestone: M6
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
SUBTOTAL with function_num 6 (and 106) over references holding no numbers is 0, as PRODUCT is.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Rule and cell: docs/excel-semantics/numbers.md "SUBTOTAL 6 over no numbers
is 0" (added by this triage). `DATE(2071,6,SUBTOTAL(6,Data!C5:C7))` over three text cells is 62609 (day 0); ours gives SUBTOTAL 1
(the empty product), so 62610. `PRODUCT` of the same range already gives 0. Code: `fn_subtotal` (crates/rustsheet-engine/src/functions/subtotal.rs);
reuse PRODUCT's empty rule, do not copy it. SUBTOTAL is an M5 function (catalog); filed at M6 with the gen-m6 triage.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 1 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/216e3c05f407ca79 Calc!A21 | `=DATE(2071,6,SUBTOTAL(6,Data!C5:C7))` | 62609.0 | 62610.0 |

## Acceptance criteria
- [ ] This 1 known-failure line disappears from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] Unit tests cover `SUBTOTAL(6,<text cells>)` and `SUBTOTAL(106,<blank cells>)` = 0.
- [ ] numbers.md's entry says Ours: implemented (T-2269).
- [ ] `make check` green; reviewers approve.

## Out of scope
- AGGREGATE's PRODUCT (probe separately if it differs).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.
