---
id: T-2344
title: "Probe a single cell, name or INDEX result holding a boolean or text as the data of QUARTILE/PERCENTILE/PERCENTRANK"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The data argument of QUARTILE*, PERCENTILE* and PERCENTRANK* has Excel-evidenced readings when it is a single cell, a defined name or an INDEX/OFFSET result holding a boolean or text.

## Context
T-1748 made a typed boolean or text a one-element array in `percentile_numbers` (percentile.rs) only when the node is not a `ReferenceKind`. Unprobed: `QUARTILE(B1,1)` with B1 = TRUE, "1", "abc"; a name pointing at such a cell; `QUARTILE(INDEX(C1:C3,2),1)`; `QUARTILE("abc",1)` (unit test only). SMALL/LARGE treat a cell holding a scalar as #NUM! (chisq.rs `numbers_from_ranked_arg`).

## Acceptance criteria
- [ ] A probes-numeric case with an oracle sidecar covers those forms; the engine matches, and the hand-written expectations in test_sorted_data_lift.rs (`typed_scalar_data_...`) cite it.
- [ ] statistics.md drops UNVERIFIED for the single-cell reading.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
