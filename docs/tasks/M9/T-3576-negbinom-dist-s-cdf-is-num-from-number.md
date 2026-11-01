---
id: T-3576
title: "NEGBINOM.DIST's cdf is #NUM! from number_s 1E+13 at 28 failures"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3553]
---

## Goal
`NEGBINOM.DIST(f, s, p, TRUE)` returns `#NUM!` wherever Excel does at a huge number_s, and the cut points are known:
probed and recorded in statistics.md.

## Context
- Found by T-3553's Gate A fill and pinned by probes-numeric/t3553-negbinom-huge-s (excel-graph, 2026-11-01).
  statistics.md has the entry: "NEGBINOM.DIST's cdf is #NUM! at a huge number_s (T-3553)".
- What the probe shows:
  - `#NUM!` at number_s 1E+13, 1E+14, 999999999999999, 1E+16 and 1E+20 with 28 failures, at p 0.1, 0.32 and 0.5.
  - 0 at number_s 1E+12 and below.
  - 0 at 0 and 14 failures with number_s 999999999999999.
  - The pmf (`FALSE`) and the legacy NEGBINOMDIST stay 0.
- The engine returns 0 at all seven `#NUM!` cells.
- Code: `crates/rustsheet-engine/src/functions/statistical/binom.rs`, the NEGBINOM.DIST kernel. Its cdf sums pmf terms in
  double-double, so Excel's cdf presumably takes another path that overflows or refuses there.
- The cuts are unknown: number_s between 1E+12 and 1E+13, failures between 14 and 28. Whether the cut depends on the
  product of the two is also unknown.

## Acceptance criteria
- [ ] A probe brackets the number_s cut and the failures cut, and tests whether the cut is a product or a sum of the two.
      statistics.md records the rule.
- [ ] The engine matches every probed cell, including probes-numeric/t3553-negbinom-huge-s (an ungated suite).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The accuracy of NEGBINOM.DIST at a large number of failures (T-3352).
