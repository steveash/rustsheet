---
id: T-2095
title: "F.DIST density at x = 0: #NUM! for df1 = 1 and 1 for df1 = 2"
milestone: M9
lane: functions-stat
status: done
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

T-2586 round 5 (2026-10-24) measured every F.DIST(.,FALSE) argument and range rule on an untouched hold-out:
x = 0 depends on trunc(d1) only (1 → #NUM!, 2 → exactly 1, 3+ → 0, whatever d2 is); a subnormal x is #DIV/0!; the
result is #NUM! exactly when fl(trunc(d1)·x) overflows; an F below 2^-1022 is 0 (statistics.md "F density is not the
beta density, R's df or a GAMMALN formula", round 5 block; docs/research/t2464/r5-f-edges/edgemodel.py `rule()`). This
task can take all of them.

## Acceptance criteria
- [x] t1049-fdist A27 and A28 pass in `make eval`. Every other t1049-fdist cell keeps its verdict,
      and no gated baseline (eval/baseline/*.txt) gains an F line.
- [x] A unit test covers `F.DIST(0,1,7,FALSE)` (`#NUM!`), `F.DIST(0,2,7,FALSE)` (1),
      `F.DIST(0,2.9,7,FALSE)` (1: df truncates) and `F.DIST(0,3,7,FALSE)` (0). The code comment cites
      A27 and A28.
- [x] statistics.md's "Density at 0" bullet names T-2095 as the implementation for F.DIST.
- [x] `make check` green; reviewers approve.

## Out of scope
- F.DIST's boolean arguments (T-1849) and its last digits (T-0970); the huge-df pdf (T-1864).

## Outcome
- `f_dist_of` (fisher.rs) returns `#NUM!` for `F.DIST(0,df1,df2,FALSE)` when the truncated df1 is 1
  and 1 when it is 2, before the pdf; df1 > 2, the cdf and F.DIST.RT at 0 are unchanged.
  probes-numeric/t1049-fdist A27 and A28 pass; the case's other failing cells are the last-digit
  ones (T-0970), unchanged, and no other suite cell takes this path. Unit test
  `test_fn_f_dist_density_at_zero`. statistics.md "Density at 0" names T-2095 for F.DIST.
- Review follow-up: probes-numeric/t2095-fdist-zero (excel-graph, 2026-10-25) confirms the rule for
  df2 = 1 and 100 and for a truncating df1 (1.5 -> `#NUM!`, 2.9 -> 1); all 6 cells pass.
