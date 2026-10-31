---
id: T-3430
title: "BETA.INV: find Excel's subnormal cut of the flipped quantile near p = 1 (depends on alpha)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
BETA.INV and BETAINV give Excel's 1 or `#NUM!` (and the lower tail's 0 or `#NUM!`) where the flipped quantile y is a
subnormal double, instead of T-3351's rule that a subnormal y is `#NUM!`.

## Context
- T-3351 (probes/t3351-beta-inv-p-near-1, excel-graph) found that BETA.INV(p, alpha, beta) for p above 1/2 is 1 - y,
  with y the lower quantile of Beta(beta, alpha) at 1 - p, and `#NUM!` once Excel's search for y fails. The failure
  is inside the subnormals and depends on alpha: with beta = 0.03, Excel gives 1 at y = 2.4E-309 (alpha 10, C199),
  1.5E-308 (alpha 2, A199) and 1.4E-317 (alpha 0.5, A211), and `#NUM!` at y = 5.6E-312 (C200), 3.4E-311 (A200) and
  3.3E-320 (A212) (y from 50-digit mpmath). The lower tail behaves the same way: `BETA.INV(6E-10,0.03,2)` is 0 (a
  subnormal shown as 0) and `BETA.INV(5E-10,0.03,2)` is `#NUM!` (B199, B200).
- For alpha 0.5 the cut sits where y^(beta-1) overflows (about 1.3E-318 at beta 0.03), which suggests Excel's
  Newton step goes non-finite there. For alpha >= 1 the cut is near 1E-310, which that does not explain.
- T-3351 treats any subnormal y as `#NUM!`. That mismatches A199, A209, A210, A211 and C199, and the lower tail's
  B199, B209, B210 and B211 (if those are not already right).
- Code: `upper_quantile_near_one` in crates/rustsheet-engine/src/functions/statistical/beta.rs.

## Acceptance criteria
- [ ] A probe that brackets the cut on a grid of alpha and beta (excel-graph), with its mechanism in
      docs/excel-semantics/statistics.md "BETA.INV where the quantile rounds to 1 (T-3351)".
- [ ] The cells above match. Any threshold chosen by scoring follows decision 0188 R4 (hold-out, fresh sweep,
      `FITTED`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Normal-range values (T-3351).
