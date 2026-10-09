---
id: T-1243
title: "BASE with a huge min_length allocates without bound (abort); Excel is #NUM!"
milestone: M5
lane: functions-math
status: todo
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
- [ ] A unit test evaluates `BASE(245,30,1E+15)` to `#NUM!` without a large allocation; the probed bound matches Excel.
- [ ] Other argument-sized allocations found by the search are bounded or filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
