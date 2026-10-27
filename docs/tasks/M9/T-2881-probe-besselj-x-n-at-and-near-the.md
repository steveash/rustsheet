---
id: T-2881
title: "Probe BESSELJ(x,n) at and near the turning point n = x (n from 1000 to 1E+5) to fit Excel's Miller recurrence rounding"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-computed values (excel-graph) for `BESSELJ(x,n)` where the Miller downward recurrence runs at or near its turning
point (n = x), enough of them to fit how Excel's recurrence rounds (T-2548).

## Context
T-2548 cannot fit `BESSELJ(1E+5,100000)` (Excel 0.009636944011336267, ours 0.00963694401133629, true value
0.00963694401133786) from one cell. Near n = x the result is very sensitive to the rounding of `tox = 2/x` and of each
recurrence step. Our double recurrence, x87 64-bit steps (stored or kept), FMA steps and the step associations already
tried (T-2548 Context and "Blocked because") scatter by about 2E-16 around Excel's value, and none of them hits it. More points are needed to
tell the models apart. The kernel is `jn_miller` in crates/rustsheet-engine/src/functions/engineering/transcendental/bessel_excel.rs.
Existing evidence: probes/m9-bessel-order-limits (`BESSELJ(1E+5,100001)` 0.0094462635180178) and
probes/t2546-bessel-past-cap.

## Acceptance criteria
- [ ] A new case `probes/t2881-besselj-turning-point` is oracle-filled (excel-graph, `rustsheet-oracle fill`), with
      about 60 cells:
      - `BESSELJ(n,n)` for n = 1000, 2000, 5000, 10000, 20000, 50000, 99999 and 100000;
      - `BESSELJ(n+d,n)` and `BESSELJ(n-d,n)` for d = 1, 10, 100 at the same orders where n - d > 0;
      - `BESSELJ(x,n)` at the non-integer x = n + 0.5;
      - a few small orders near x (`BESSELJ(84,84)`, `BESSELJ(84,83)`, `BESSELJ(100,100)`).
- [ ] Every failing cell has a baseline entry, owned by T-2548 (`make eval-baseline`); no other baseline changes.
- [ ] docs/excel-semantics/engineering.md cites the case next to the T-2548 "Not fitted" note.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting the model (T-2548, opus).
