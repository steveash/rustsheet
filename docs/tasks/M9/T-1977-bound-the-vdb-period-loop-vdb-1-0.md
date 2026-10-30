---
id: T-1977
title: "Bound the VDB period loop (VDB(1,0,1e12,0,1e12) walks 1e12 periods)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VDB cannot be made to loop for an unbounded number of periods by an untrusted formula.

## Context
Found by the T-1764 security review: `vdb_total_depr` loops once per integer period up to end_period, bounded only by life. Prefer a closed form or a cap (a cap needs a DEVIATION entry, decision 0014). See T-1820.

## Acceptance criteria
- [x] VDB(1,0,1e12,0,1e12) returns promptly, with a unit test
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
T-1820 already bounded the loop: VDB refuses an `end_period` past `MAX_DEPRECIATION_PERIODS` (100,000) with `#NUM!`
after Excel's own checks (DEVIATION "DB and VDB period cap" in docs/excel-semantics/financial.md). This task adds the
unit test `fn_vdb_huge_period_is_capped`: `VDB(1,0,1e12,0,1e12)` returns `#NUM!` at once in both modes, and so does a
one-period window at the end. A closed form would lift the cap, but it would have to reproduce Excel's charge-by-charge
summation to the bit (T-1975, T-2185), so it was not attempted. Eval delta: none (unit test only).
