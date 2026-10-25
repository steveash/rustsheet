---
id: T-2647
title: "F.DIST pdf at a tiny x: Excel's last digits where Y = density·x is subnormal (probes-numeric t1864-f-pdf-tiny-x)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1864]
---

## Goal
F.DIST's pdf at a tiny x gives Excel's value to 15 digits where Excel's stored Y = density·x is subnormal:
probes-numeric t1864-f-pdf-tiny-x A5, A6, A10 and A17.

## Context
Found by T-1864's edge probes (probes-numeric t1864-f-pdf-tiny-x, t1864-f-pdf-edges, excel-graph, 2026-10-25).
T-1864 implements round 5's range steps (docs/excel-semantics/statistics.md, the T-2513 F density entry: Excel stores
Y = density·x, takes F = fl(Y/x), and an F below 2^-1022 is 0) in `fisher.rs` `excel_density_range`, re-forming a
subnormal Y as fl(fl(density·x)/x). That gives Excel's large subnormal errors (A6
`F.DIST(1E-215,3,1000,FALSE)`: Y = 13·2^-1074, 2.1% below the exact density; A17 `F.DIST(1E-65,10,1000,FALSE)`:
Y = 3·2^-1074, 11.6% above), but Excel's value is not exactly fl(Y/x):
- A6: Excel 6.42285339593616E-108, ours (13·2^-1074)/1E-215 = 6.422853395936205E-108 (7E-15 relative).
- A17: Excel 1.4821969375236873E-258, ours 1.4821969375237397E-258 (3.5E-14).
- A10 `F.DIST(1E-160,4,1000,FALSE)`: Excel 4.007860519064344E-160, ours 4.007860519064192E-160.
- A5 `F.DIST(1E-210,3,1000,FALSE)`: Excel 2.0745191673458065E-105, ours 2.0745191673458212E-105.
Excel's F·x/2^-1074 is 12.99999999999991 at A6, so its Y/x step is not a plain double division of the stored
subnormal (an extended-precision Y, a division by a rounded 1/x, or the EXP of a log difference are candidates).
Probe cells with a known subnormal Y (x a power of two, so Y/x is exact in double) to tell them apart.

## Acceptance criteria
- [ ] probes-numeric t1864-f-pdf-tiny-x A5, A6, A10, A17 are within 15 digits of Excel (or bit-exact), and no
      passing F.DIST cell of any suite changes.
- [ ] The rule is recorded in docs/excel-semantics/statistics.md ("F density at large degrees of freedom (T-1864)").
- [ ] `make check` green; reviewers approve.

## Out of scope
- The bit-level last ulp at a huge df (T-2642); Excel's large-shape noise (T-2586).
