---
id: T-1873
title: "Bound HYPGEOM.DIST's cdf sum, then lift HYPGEOM.DIST and HYPGEOMDIST (SPEC-0011)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
HYPGEOM.DIST and HYPGEOMDIST lift every argument over arrays and ranges like the other statistical functions, once
HYPGEOM.DIST's cdf sum has a per-call cap of its own.

## Context
Split from T-1848 (statistics.md "Quantile searches: per-call cap and lifting"). statrs 0.18's `Hypergeometric::cdf`
sums the pmf term by term from 0 to x, three `ln_binomial`s a term:
`HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)` takes about 4 s on main. Terms below `max(0, n + K - N)` are
exactly 0 and terms far from the mode underflow to 0, so a bound that keeps today's values may skip them; past that a
term cap is a DEVIATION. The pmf (`cumulative` FALSE, HYPGEOMDIST) is closed form. HYPGEOMDIST builds a fifth argument
node today (`fn_hypgeomdist_compat`); a kernel takes the four values instead.
Evidence: probes/stat-argument-modes H330-H342 (every argument over `{v;v}`, the first over a range);
probes/stat-quantile-lift H51-H54 (the last argument over a two-cell range is 2 rows, values in I and J; excel-graph,
2026-10-18). Probe a large population first if a cap changes a result.

## Acceptance criteria
- [ ] HYPGEOM.DIST's cdf has a per-call cap of its own and is entered as `Weighted` with a release measurement of its
  worst case in kinds.rs; HYPGEOMDIST is `Constant` if its kernel is the closed-form pmf; their lift cells of
  probes/stat-argument-modes and probes/stat-quantile-lift H51-H54 pass.
- [ ] A test shows the worst-case scalar call is bounded, with the values probes-numeric and ironcalc-calc check unchanged.
- [ ] A cap that changes a result is a DEVIATION entry in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
