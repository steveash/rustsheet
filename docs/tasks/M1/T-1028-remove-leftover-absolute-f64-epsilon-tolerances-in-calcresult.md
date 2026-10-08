---
id: T-1028
title: "Remove leftover absolute f64::EPSILON tolerances in CalcResult Ord and criteria matching"
milestone: M1
lane: engine
status: done
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
Criteria equality (=, <>, blank-as-0) and `Ord for CalcResult` now compare 15-digit values instead of an absolute `f64::EPSILON`. New probe `probes/tiny-vs-zero-criteria` (oracle-filled) pins Excel: `COUNTIF(r,0)` ignores 1E-17, `COUNTIF(r,1)` matches 1+2^-52. Eval: the new probe passes in full (all cells); baseline known failures unchanged (230), no regressions. Quirk recorded in `docs/excel-semantics/numbers.md`.
