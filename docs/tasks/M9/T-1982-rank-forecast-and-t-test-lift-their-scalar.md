---
id: T-1982
title: "RANK, FORECAST and T.TEST lift their scalar arguments over a whole data argument"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: [T-1813]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
Done here: FORECAST/FORECAST.LINEAR `x` and T.TEST/TTEST `tails`/`type` on the driver, and the
order-statistic precedence. RANK moved to **T-1999**, which carries the RANK findings below.
As written at the split from T-1813:
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
- Domain checks are not uniform: T.TEST's `tails` 3 beats the data's error (P88), Z.TEST's
  `sigma <= 0` does not (stat-test-lift-b P93). Probe each kernel's domain checks.
- SMALL, PERCENTILE, TRIMMEAN and QUARTILE.EXC of `NA()` with a bad `k` are `#N/A` (P80-P83;
  ours gives `#VALUE!` for P81-P83).

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

## Outcome
FORECAST and FORECAST.LINEAR (`statistical/forecast.rs`) read the data once into the regression
line (`forecast_line`), and `x` lifts. T.TEST and TTEST (`statistical/t_dist.rs`) read the samples
once, and `tails`/`type` lift: an element checks `tails`, then `type`, then takes the samples'
error, and is charged the samples' cells (`LiftCost::Cells`). The test itself is `t_test_p`, with
empty-message errors. All four functions are in `Function::arg_kinds` (`ONE_THEN_TWO_DATA`,
`TWO_DATA_THEN_2`) and in `lifting_table_functions!`. Their fallback signature and shape arms are
deleted, `fn_forecast_linear` is merged into `fn_forecast`, and `eval/catalog/lift-kinds.json` is
regenerated.

PERCENTILE*/QUARTILE*/PERCENTRANK*'s reader and TRIMMEAN's reader now return an error value as
that error, so the data's error comes before a bad `k` (P81-P83). PROB keeps `collect_numeric_flat`'s
`#VALUE!`, since it was not probed with an error data argument. FREQUENCY's and PERCENTILE's readers now
delegate to the new `numeric_flat_of`, PROB's reader, instead of copying it. Duplication drops from 3630 to 3619; the
ratchet file is left for the steward, since changing it needs a decision record.

RANK needs a design for its two-kind `ref` and is split to T-1999. The unprobed orders the
fidelity review listed are T-2000.

Eval delta (probes): fixed 32, regressions 0, new cases 0.
- stat-test-lift: P1-P4, P6-P9, P60-P62, P74, P81-P83, P88.
- stat-argument-modes: H260, H261, H264, H265, H537-H539, H543-H545.
- regression-forecast: L7, M7, P7, Q7, L13, P13 (FORECAST's lifted `x`).

Still failing, as the task scoped them: P64-P67 (T.TEST's value is NumericClose, T-0969), and RANK (T-1999).
