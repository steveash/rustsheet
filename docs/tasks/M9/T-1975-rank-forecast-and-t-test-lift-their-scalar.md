---
id: T-1975
title: "RANK, FORECAST and T.TEST lift their scalar arguments over a whole data argument"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-1813]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
The rest of T-1813: RANK/RANK.EQ/RANK.AVG `number` and `order`, FORECAST/FORECAST.LINEAR `x` and
T.TEST/TTEST `tails`/`type` lift over an array and a multi-cell range through the SPEC-0011
driver, with the data arguments read once by the function's own reader; and SMALL, PERCENTILE,
TRIMMEAN and QUARTILE.EXC return the data's error before a bad `k`.

## Context
Split from T-1813, which put Z.TEST and PROB on the driver (`functions/statistical/z_test.rs`,
`prob.rs`: data `WholeByCaller`, read once, the scalar arguments checked before the data's error).
Evidence: probes/stat-test-lift (excel-graph, T-1813) and probes/stat-argument-modes H260, H261,
H264, H265, H492, H493, H495, H496, H498, H499, H537-H539, H543-H545; statistics.md
"RANK, FORECAST, PROB, Z.TEST and T.TEST: lifted scalar arguments".
- An element checks its scalar arguments before the data's error: `FORECAST("a",D1:D3,A1:A3)` is
  `#VALUE!` and `T.TEST(D1:D3,A1:A3,3,2)` `#NUM!` with D2 = `#N/A` (P88, P89), while an element
  with good arguments is the data's error (P4, P63). T.TEST truncates `tails` and `type`, and
  `tails` 3 or `type` 4 is `#NUM!` per element (P60, P61, P65).
- RANK: a lifted `number` against a non-reference `ref` broadcasts with the ref's shape, every
  element `#VALUE!` (`RANK({4;"x"},IF(A1:C1>0,A1:C1))` is 2x3, P28-P31). `rank_static_result`
  gives that shape today; entering RANK in the table drops it, so `lifted_static_result` must
  keep it (or RANK keeps a fallback arm by decision). `RANK({1;3},D1:D3)` is `#N/A` (P25: the
  data's error, not the rank of 1).
- A 1x1 array argument gives a 1x1 array: `TYPE(RANK({4},A1:A8))`, `TYPE(FORECAST({4},..))` are
  64 (P72, P74).
- SMALL, PERCENTILE, TRIMMEAN and QUARTILE.EXC of `NA()` with a bad `k` are `#N/A` (P80-P83;
  ours gives the `k` error for P81-P83).

## Acceptance criteria
- [ ] probes/stat-test-lift P1-P9, P20-P33, P60-P67, P72, P74, P81-P83, P88 and the
  stat-argument-modes cells above leave eval/baseline/probes.txt, with no new F lines.
- [ ] RANK*, FORECAST* and T.TEST/TTEST are in `Function::arg_kinds` and
  `lifting_table_functions!`, and their fallback arms are deleted (or a decision keeps RANK's).
- [ ] Each kernel's check order is the probed one, and its element errors carry empty messages.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The pairing functions' unequal sizes (T-1292); FORECAST.ETS (T-1293, T-1553).
- Cells that miss by ulps today (T.TEST H535-H546) stay with their accuracy tasks.
