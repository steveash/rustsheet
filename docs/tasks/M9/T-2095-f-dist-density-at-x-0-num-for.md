---
id: T-2095
title: "F.DIST density at x = 0: #NUM! for df1 = 1 and 1 for df1 = 2"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`F.DIST(0,df1,df2,FALSE)` gives Excel's density at zero: `#NUM!` when the truncated df1 is 1 (the
density is infinite there) and exactly 1 when it is 2. Today both give 0.

## Context
Gap triage 2026-10-20. docs/excel-semantics/statistics.md, "Thin distribution functions ... (T-1049)",
"Density at 0", says: "`F.DIST(0,1,7,FALSE)` and `CHISQ.DIST(0,1,FALSE)` are `#NUM!` (the density is
infinite), the df=2 value is 1 (F) and 0.5 (CHISQ), higher df give 0". It also says T-2035
implements these edges, but T-2035 only lists the HYPGEOM.DIST, WEIBULL.DIST and PHI cells. The
CHISQ.DIST cells already pass, so only F.DIST is left.
Failing cells (excel-graph; probes-numeric is not gated, so read them with
`make eval-show CASE=probes-numeric/t1049-fdist.case.json`):
- probes-numeric/t1049-fdist Calc!A27 `F.DIST(0,1,7,FALSE)`: Excel `#NUM!`, ours 0.
- probes-numeric/t1049-fdist Calc!A28 `F.DIST(0,2,7,FALSE)`: Excel 1, ours 0.
Code: `f_dist_of` and `f_pdf` in crates/rustsheet-engine/src/functions/statistical/fisher.rs. df1 is
already truncated before the pdf. For x == 0 and not cumulative: return `#NUM!` for df1 == 1 and 1.0 for
df1 == 2. Keep today's 0 for df1 > 2: it matches the doc and is the mathematical value. A cumulative
F.DIST at 0 and F.DIST.RT at 0 do not change (`F.DIST.RT(0,...)` is 1 per the doc).

## Acceptance criteria
- [ ] t1049-fdist A27 and A28 pass in `make eval`. Every other t1049-fdist cell keeps its verdict,
      and no gated baseline (eval/baseline/*.txt) gains an F line.
- [ ] A unit test covers `F.DIST(0,1,7,FALSE)` (`#NUM!`), `F.DIST(0,2,7,FALSE)` (1),
      `F.DIST(0,2.9,7,FALSE)` (1: df truncates) and `F.DIST(0,3,7,FALSE)` (0). The code comment cites
      A27 and A28.
- [ ] statistics.md's "Density at 0" bullet names T-2095 as the implementation for F.DIST.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F.DIST's boolean arguments (T-1849) and its last digits (T-0970); the huge-df pdf (T-1864).
