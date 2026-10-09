---
id: T-1152
title: "Probe LARGE/SMALL/RANK.AVG/FREQUENCY over unions: overlap, error cells, area order, SMALL(IF/INDEX/3-D)"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles what the engine assumes (from SUM) for LARGE, SMALL, RANK.AVG and FREQUENCY over a union.

## Context
T-1068 review (excel-fidelity, minor). Unit test `order_statistics_take_a_union`
(test_multi_area_arguments.rs) holds expectations with no Excel result behind them: overlapping areas count twice
(`LARGE((A1:B2,B2),2)` = 5), an error cell in an area is the result (`LARGE((A1,E1),1)`, `SMALL((A1,E1),1)`),
`RANK.AVG(5,(A1,B2,C3),1)` = 2 and a FREQUENCY union holding an error cell. The existing probes use 1, 5, 7, so they
cannot show the order areas are walked in. SMALL now reads its argument like LARGE (`numbers_from_reference_arg`), so
`SMALL(IF(..))`, `SMALL(INDEX(..))`, a 3-D `SMALL(Sheet1:Sheet2!A1,1)` and `RANK(.., IF(..))` are also unprobed.
Write a probe next to probes/multiarea-rank with the excel-probe skill and fill it with the oracle.

## Acceptance criteria
- [x] A probe case with oracle values covers the forms above (use values whose order matters, e.g. 7, 1, 5); references.md records each finding, and a fix task is filed for any engine mismatch.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/multiarea-order-stats (oracle-filled, excel-graph). The engine already matched Excel on 30 of 33 scored cells: overlap counts twice, area order is irrelevant, a text cell in a union is ignored while a real error cell propagates, SMALL(IF/INDEX/3-D) work. The one mismatch (RANK of an array ref argument is `#VALUE!`) is T-1203. probes baseline updated (3 known-failure cells).
