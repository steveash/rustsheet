---
id: T-3075
title: "gen-m2 lookup top-up batch: oracle-filled VLOOKUP, HLOOKUP and INDEX cells that find and pick (after T-3074)"
milestone: M2
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3074]
---

## Goal
gen-m2 holds a seeded, excel-graph-filled batch in which most VLOOKUP, HLOOKUP and INDEX outer calls return a found
value rather than an error, so the gate's M2 lookup evidence covers the search and the pick themselves.

## Context
- Before T-3074, 87% of VLOOKUP and HLOOKUP and 77% of INDEX outer-call cells in gen-m2 are errors (T-3074 Context),
  so gen-m2's 100% says little about their matching. T-2719 ran the same top-up for SUMIFS, COUNTIFS and MATCH
  (`--milestone M2 --focus --functions SUMIFS,COUNTIFS,MATCH --count 2 --formulas 300 --seed 2719`, two cases).
- Commands: `cd tools && uv run rustsheet-gen --milestone M2 --focus --functions VLOOKUP,HLOOKUP,INDEX --count 2
  --formulas 300 --seed 3075`, then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m2`, then `make eval`
  and `make eval-baseline` (no regressions allowed).

## Acceptance criteria
- [ ] Two new gen-m2 cases with excel-graph sidecars (canary-verified); in them at most 35% of the VLOOKUP, HLOOKUP
      and INDEX outer-call cells are errors, counted the way T-3074's Context counts them and reported in the Outcome.
- [ ] gen-m2 stays at 100% (no visibly-off cell); any new-coverage failure is a known failure owned by a filed M2 task
      named in the Outcome, and `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Generator changes (T-3074). Engine fixes for any mismatch found (file them).
