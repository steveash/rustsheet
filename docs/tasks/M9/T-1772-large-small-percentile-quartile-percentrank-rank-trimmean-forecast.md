---
id: T-1772
title: "LARGE, SMALL, PERCENTILE, QUARTILE, PERCENTRANK, RANK, TRIMMEAN, FORECAST, PROB, Z.TEST and T.TEST lift their scalar arguments over a whole data argument"
milestone: M9
lane: functions-stat
status: todo
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

## Out of scope
- The pairing functions' unequal sizes (T-1292). The data rules themselves, such as which cells LARGE skips.
