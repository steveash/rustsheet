---
id: T-3505
title: "GAMMA.INV returns #NUM! at shapes near 8E6 where Excel returns a number"
milestone: M9
lane: functions-stat
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV return Excel's number, not `#NUM!`, at the shapes below the
huge-shape band (1E9, T-3070) where the engine gives up today.

## Context
- Held-out blind sweep (excel-graph, filled 2026-10-31): `GAMMA.INV(0.5,7990464.81598859,1)` is #NUM! in the engine
  and 7990464.482655259 in Excel. A wrong type is a semantic failure. The other 135 GAMMA.INV cells of the sweep at
  shapes 20 to 1E9 with p from 0.01 to 0.5 were fine.
- The shape lies below `HUGE_SHAPE_MIN` (incomplete_gamma/huge_shape.rs), so the cause is in `gamma_quantile`
  (incomplete_gamma/mod.rs): suspects are the one `MAX_TERMS` term budget its cdfs share, the bracket, or another
  directive-5 cap.
- The failing region (shape range, p range, the chi-squared functions at df/2) is measured locally and recorded here
  before the fix.

## Acceptance criteria
- [ ] The failing region is measured and recorded (shape and p ranges, per function).
- [ ] A probe case under eval/suites/probes/cases/ (tag M9), filled by the oracle (excel-graph), covers the region and
      points on both sides of it.
- [ ] Every cell of that case has Excel's type, and its numbers are within 15 significant digits of Excel's (decision
      0083's rule), or the residue is listed.
- [ ] Per-call work stays bounded (directive 5): no cap removed.
- [ ] The finding is recorded in docs/excel-semantics/statistics.md.
- [ ] No gated regression and no cell newly visibly off (decisions 0083, 0111); main-vs-HEAD per-cell comparison in
      the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The huge-shape band from 1E9 (T-3070) and the last-digit work of T-3269, T-3285 and T-3311.
