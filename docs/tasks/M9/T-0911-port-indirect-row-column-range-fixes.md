---
id: T-0911
title: "Port IronCalc INDIRECT (R1C1) and ROW/COLUMN-on-range fixes"
milestone: M9
lane: functions
status: done
priority: P3
size: M
depends_on: []
---

## Goal
Upstream fixed INDIRECT reading R1C1 references (`b65478c8`, `72afa4a0`) and ROW/COLUMN
accepting ranges (`b6d34abc`). Evaluate each against Excel and port what matches.
Split: this task covers ROW/COLUMN only; INDIRECT and the triage list moved to T-0918.

## Acceptance criteria
- [x] ROW/COLUMN of a multi-cell reference return a vector; evidence is the Excel-authored
  the upstream `ROW_*` workbook (LOOKUP_AND_REFERENCE) cached values (21 new checked cells, all pass).
- [x] Spec/unit cases added (`test_fn_row_column.rs`, implicit-intersection cases); `make check` green.
- [x] Commit carries `Ported-From: ironcalc@b6d34abc`.

## Outcome
Ported `b6d34abc` (ROW/COLUMN on ranges incl. static analysis for implicit intersection).
ironcalc-calc checked cells 71390 -> 71411 with 0 new failures. Remaining work filed as T-0918.
