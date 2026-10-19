---
id: T-1813
title: "RANK, FORECAST, PROB, Z.TEST and T.TEST lift their scalar arguments over a whole data argument"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-1772]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The rest of T-1772: RANK/RANK.EQ/RANK.AVG `number` (and `order`), FORECAST/FORECAST.LINEAR `x`, PROB `lower_limit`/`upper_limit`, Z.TEST/ZTEST `x`/`sigma` and T.TEST/TTEST `tails`/`type` lift over an array and over a multi-cell range through the SPEC-0011 driver, with the data arguments read once by the function's own reader and not lifted.

## Context
Split from T-1772, which entered LARGE, SMALL, PERCENTILE*, QUARTILE*, PERCENTRANK* and TRIMMEAN (`functions/statistical/sorted_data.rs`: the data argument is `ArgKind::WholeByCaller`, read by the function's existing node-based reader, and the call is charged `LiftCost::Prepared` or `Cells`). Evidence: probes/stat-argument-modes H260, H261, H264, H265, H488, H489, H492, H493, H495, H496, H498, H499, H537-H539, H543-H545, H566, H567, H571, H572 (`ROWS(..)` = 2 over `{v;v}` and over G1:G2), today `#N/IMPL!`/`#VALUE!`.
- RANK: `rank_static_result` gives a non-reference `ref` its array shape (statistics.md "RANK of a non-reference array"). Entering RANK in the table drops that fallback arm, so `lifted_static_result` must keep that shape, or the RANK entry must wait for a way to express it. `rank_number_error` spills a bad `number` over a non-reference `ref`'s shape; decide (probe) what a lifted element does there.
- FORECAST, PROB and T.TEST read two data arguments (pairing: T-1292 owns unequal sizes). Their readers use `evaluate_node_in_context`, so a single-cell reference is its value there: keep that by reading them as `WholeByCaller`.
- Z.TEST's `x` is `#VALUE!` for text and booleans today (`evaluate_node_in_context` + a `Number` match), and statistics.md says text "5" and TRUE coerce: check the evidence before keeping the kernel's check.
- Probe first (excel-probe skill, one case): per-element errors, a data error beside a lifted argument, RANK's `order` and Z.TEST's `sigma` over an array and a range, PROB's `upper_limit`; and T-1772's unprobed 1x1 array argument: `=TYPE(LARGE(A1:A8,{1}))` and `=TYPE(PERCENTRANK(A1:A8,4.5,{1}))` (ours 64, a 1x1 array; statistics.md "LARGE, SMALL, PERCENTILE, QUARTILE, PERCENTRANK and TRIMMEAN: each lifted element is the scalar call"). Also T-1772's inferred precedence of a data error over a bad k for SMALL, PERCENTILE, TRIMMEAN and QUARTILE.EXC (only LARGE and PERCENTRANK are evidenced, gen-m9): e.g. `=SMALL(NA(),0)`, `=PERCENTILE(NA(),2)`, `=TRIMMEAN(NA(),2)`, `=QUARTILE.EXC(NA(),9)`.
- Cells that miss by ulps today (T.TEST H535-H546, Z.TEST H564-H573) stay with their accuracy tasks.

## Acceptance criteria
- [ ] The lift cells above leave eval/baseline/probes.txt, with no new F lines.
- [ ] The functions are in `Function::arg_kinds` and `lifting_table_functions!`, and their fallback arms are deleted.
- [ ] Each kernel's check order is the scalar function's, and its element errors carry empty messages.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The pairing functions' unequal sizes (T-1292); FORECAST.ETS (T-1293, T-1553).
