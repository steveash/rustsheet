---
id: T-1772
title: "LARGE, SMALL, PERCENTILE, QUARTILE, PERCENTRANK, RANK, TRIMMEAN, FORECAST, PROB, Z.TEST and T.TEST lift their scalar arguments over a whole data argument"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The scalar arguments that sit next to a data array lift over an array and over a multi-cell range through the SPEC-0011 driver: LARGE/SMALL `k`, PERCENTILE* and QUARTILE* `k`/`quart`, PERCENTRANK* and Z.TEST `x`, RANK* `number`, TRIMMEAN `percent`, FORECAST(.LINEAR) `x`, PROB `lower_limit` and T.TEST `tails`/`type`. The data argument is read once and is not lifted. This task is the spec's first real `Whole` + `LiftCost::Prepared` consumer.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md. Evidence: probes/stat-argument-modes (statistics.md "Statistical function argument modes"). These functions' lift cells are failing today, `#N/IMPL!` for an array and `#VALUE!` for a range.
- The data argument is `Whole`, or `WholeByCaller` where it needs the aggregate reading (SPEC-0009's `evaluate_aggregate_arg`, walked with `walk_aggregate_read`). The kernel sorts or collects it on its first element and keeps the result in its closure, and the call declares `LiftCost::Prepared(cells)` (cells + elements). So `LARGE(A1:A100000,SEQUENCE(1000))` sorts once, not 1000 times, and is not refused.
- Split from T-1290, which takes the stat functions whose every argument is scalar.
- FORECAST.ETS stays `#N/IMPL!` (T-1293, T-1553).

## Acceptance criteria
- [ ] The probes/stat-argument-modes lift cells of these functions leave eval/baseline/probes.txt, with no new F lines. Cells that still miss by ulps are named in the Outcome with their owner task.
- [ ] A unit test shows the data argument is read and prepared once per call, through a kernel that counts its prepare step.
- [ ] `=ROWS(LARGE(A1:A1000000,SEQUENCE(200)))` over a filled column runs (`Prepared`), and a call above the cap is `#ERROR!` before the first element.
- [ ] The functions are in `Function::arg_kinds`, and their old static-analysis arms are deleted.
- [ ] `make check` green; reviewers approve.
- [ ] The entries it adds make the variant live, and it removes the `#[allow(dead_code)]` on `ArgKind::Whole` and `LiftCost::Prepared` (functions/lift/kinds.rs, driver.rs), and on `ArgKind::WholeByCaller` if this task enters a caller-read argument (else name the task that will) (T-1770 left the allow with a `TODO(T-1772)`).

## Out of scope
- The pairing functions' unequal sizes (T-1292). The data rules themselves, such as which cells LARGE skips.

## Outcome
Scope narrowed to the sort-once functions (size M): LARGE, SMALL, PERCENTILE/PERCENTILE.INC/.EXC, QUARTILE/QUARTILE.INC/.EXC, PERCENTRANK/PERCENTRANK.INC/.EXC and TRIMMEAN. RANK*, FORECAST(.LINEAR), PROB, Z.TEST/ZTEST and T.TEST/TTEST moved to **T-1810** (RANK's non-reference `ref` shape needs a design in `lifted_static_result`; the others read two data arguments).

- **Probe:** probes/stat-whole-lift (48 cells, excel-graph, 2026-10-17): each lifted element is the scalar call (k out of range, fractional k, typed and referenced elements, a data error or empty data in every element, 2-D and row × column shapes, `significance` lifts over an array and a range). Recorded in statistics.md "LARGE, SMALL, PERCENTILE, QUARTILE, PERCENTRANK and TRIMMEAN: each lifted element is the scalar call".
- **Code:** `functions/statistical/sorted_data.rs` (`lift_with_sorted_data`): count check, the data argument read once by the function's own reader, sorted on the first element, kernels over the sorted slice. Table entries `DATA_THEN_1` / `DATA_THEN_1_OF_2` (`WholeByCaller`, then `Lift`); fallback signature and shape arms deleted. New `Model::lift_call_costed`; LARGE/SMALL/PERCENTILE*/QUARTILE*/PERCENTRANK* are charged `LiftCost::Prepared(numbers)`, TRIMMEAN `Cells(numbers)` (each element averages the kept values). Kernel errors carry empty messages.
- **Deviation from the criterion:** the data argument is `WholeByCaller`, not `Whole`: the existing readers depend on the argument's node (a cell reference vs a typed scalar, unions, external ranges, `evaluate_node_in_context` for one cell), which a `Whole` value cannot tell apart, so a `Whole` read would change scalar results. `#[allow(dead_code)]` removed from `WholeByCaller` and `LiftCost::Prepared`; `Whole` keeps it with `TODO(T-1236)` (WORKDAY's holidays). SPEC-0011 has an "As built (T-1772)" note.
- **Tests:** test/statistical/test_sorted_data_lift.rs (the probe cells, unlifted values and AGGREGATE 14/17, `ROWS(LARGE(A1:A1000000,SEQUENCE(200)))` = 200 over a filled column, a 1.1M-cell result `#ERROR!`, TRIMMEAN's per-element charge, empty messages); `sorted_data::tests` counts one read and one sort for four elements.
- **Eval:** probes 24 fixed (stat-argument-modes H345-H346, H350-H351, H422-H423, H427-H428, H432-H433, H437-H438, H442-H443, H447-H448, H452-H453, H457-H458, H462-H463, H549-H550), 0 regressions, no changed failing value in any suite (cell-by-cell against main). Still failing in these functions: H424 and H429 (`PERCENTILE(C1:C6,0.3)` is 1.9, Excel 1.9000000000000001: an ulp miss, T-1811).

- **Review round:** the directive-5 DEVIATION (numbers + elements, TRIMMEAN numbers × elements, the 2^27 cap and the array bound) is in statistics.md and arrays.md; the 1x1 array argument is marked unprobed (T-1810 probes it). Kernels check their own arguments before they sort (`SortedNumbers::sorted`), so `=LARGE(A:A,0)` never sorts; a data read error keeps its precedence over k and is copied per element with an empty message. PERCENTRANK refuses a NaN x and reads its neighbours with `get`. Bench follow-up: T-1812.
