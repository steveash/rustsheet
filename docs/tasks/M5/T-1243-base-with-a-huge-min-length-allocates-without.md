---
id: T-1243
title: "BASE with a huge min_length allocates without bound (abort); Excel is #NUM!"
milestone: M5
lane: functions-math
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
BASE never allocates more than Excel's limit for `min_length`; `BASE(245,30,1E+15)` is `#NUM!` as in Excel, not a process abort.

## Context
Found by T-1238's seeded gen-m6 sample (`rustsheet-gen --milestone M6 --count 25 --seed 17`, case c73724313f573668,
not committed): `=UPPER((Data!$C9&BASE(245,30,1000000000000000.0)))` made `rustsheet-eval run` abort with "memory
allocation of 1000000000000000 bytes failed" in `Model::fn_base` (functions/math_and_trigonometry/mathematical.rs,
the `min_length` padding). excel-graph gives `#NUM!` for that cell. This kills the process on untrusted input (CLAUDE.md
directive 5). Probe the bound with the excel-probe skill (`BASE(1,2,255)`, `(1,2,256)`, `(1,2,255.9)`, `(1,2,-1)`;
Microsoft documents min_length up to 255), then reject beyond it before allocating. Also check DEC2BIN/DEC2HEX/DEC2OCT
`places`, TEXT/REPT-like padding and other functions that size a buffer from an argument (rg `repeat(`, `with_capacity(`).

## Acceptance criteria
- [x] A unit test evaluates `BASE(245,30,1E+15)` to `#NUM!` without a large allocation; the probed bound matches Excel.
- [x] Other argument-sized allocations found by the search are bounded or filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe probes/base-min-length (14 cells, excel-graph, 2026-10-09): `min_length` truncates and may be at most 255
(`BASE(1,2,256)` and `BASE(245,30,1E+15)` are `#NUM!`). `fn_base` now returns `#NUM!` past 255 before padding, so it never
allocates more than 255 characters; unit test test_base.rs. Search for other argument-sized buffers: the DEC2BIN/BIN2HEX
family already caps `places` at 10 (engineering/number_basis.rs); the remaining `with_capacity` calls are sized by data
already in memory. Recorded in numbers.md "BASE min_length is at most 255". Eval delta: probes +1 case, 14 cells, all pass.
