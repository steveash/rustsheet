---
id: T-1995
title: "MIRR: probe both powers overflowing with no positive flow, and a power underflowing to zero"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
MIRR's two remaining unprobed corners are evidenced by excel-graph and the engine matches them.

## Context
T-1798 left `MIRR({-1,-2,-3},1E+200,1E+200)` (both powers overflow, no positive flow; engine `#NUM!` because the reinvest check runs first) and a rate in (-1,0) over many flows whose running power underflows to 0 (engine `#DIV/0!` via the zero-power guard, e.g. rate -0.999999 with 400 flows) unprobed. See docs/excel-semantics/financial.md "MIRR error and edge cases", `fn_mirr` in functions/financial/mirr.rs.

## Acceptance criteria
- [ ] a probe of both cases, filled by excel-graph, with mismatches fixed in `fn_mirr` and recorded in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
