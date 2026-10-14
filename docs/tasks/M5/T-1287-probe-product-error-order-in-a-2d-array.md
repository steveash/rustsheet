---
id: T-1287
title: "Probe PRODUCT error order in a 2D array and a no-number array with a scalar"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PRODUCT error order over a 2D array and a no-number array combined with a scalar match Excel.

## Context
Left open by the T-1165 fidelity review: probes/product-array-elements only has one-row arrays. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Oracle values for `PRODUCT({1,#N/A;#DIV/0!,2})` and `PRODUCT({"a"},3)`; engine matches or is fixed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
New probe probes/product-array-2d-and-scalar (19 cells, excel-graph): a 2D array's first error is row-major; a no-number array with a scalar gives the scalar; an in-array error ranks like a referenced error (a direct scalar error beats it, otherwise the leftmost wins), as SUM does. The engine returned an in-array error at once, so `PRODUCT({1,#N/A},#NUM!)` gave #N/A (Excel #NUM!); `fn_product` now defers it. Recorded in coercion.md. Eval: 19 new passing probe cells, 0 regressions. The same rule for other aggregates' array arguments was not probed here.
