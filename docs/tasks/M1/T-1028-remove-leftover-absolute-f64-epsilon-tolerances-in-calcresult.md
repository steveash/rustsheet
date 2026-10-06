---
id: T-1028
title: "Remove leftover absolute f64::EPSILON tolerances in CalcResult Ord and criteria matching"
milestone: M1
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
No comparison path equates a tiny nonzero value with 0 through an absolute epsilon.

## Context
Follow-up of T-0989 (review finding). `impl Ord for CalcResult` (calc_result.rs ~65) and criteria matching (functions/criteria.rs ~72, 81, 123, 251) still use `f64::EPSILON` absolute tolerances. Check where Ord is used (sort, MATCH, XLOOKUP) and whether it should delegate to `compare_values`. Needs oracle evidence (excel-probe skill) for COUNTIF-style criteria with tiny values before changing criteria.

## Acceptance criteria
- [ ] A probe records Excel's behavior for tiny values vs 0 in sort/MATCH and COUNTIF criteria; the engine matches or the gap is baselined.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Criteria equality is exact and `Ord for CalcResult` uses the 15-digit rule; probe tiny-vs-zero-criteria added. Probe cells fixed: see baseline.
