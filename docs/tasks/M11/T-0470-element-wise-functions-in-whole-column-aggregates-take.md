---
id: T-0470
title: "Element-wise functions in whole-column aggregates take the tailed path (SPEC-0020)"
milestone: M11
lane: perf
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-0469]
---

## Goal
The element-wise functions that real workbooks use inside whole-column aggregates (ISNUMBER, ISBLANK,
ISERROR, NOT, LEN, IFERROR, ...) take SPEC-0020's tailed path too, with bit-identical results.

## Context
SPEC-0020 "Which expressions are tailed": each function is added only with an argument whose cells
past the used area read the same in every row. Make the list from a scan of the fetched suites
(corpus-bench, calipers) for aggregates over whole-column expressions.

## Acceptance criteria
- [ ] The scan and the resulting list in the Outcome; each listed function in `tailed_shape` and
      `evaluate_tailed` through its existing element kernel.
- [ ] Equivalence tests per function (as T-0469's).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Position- or call-dependent functions (ROW, COLUMN, SEQUENCE, RAND, INDEX, OFFSET).
