---
id: T-0239
title: "GCD/LCM accept arguments up to 2^53 (Excel gives GCD(2^53,2)=2)"
milestone: M2
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
GCD and LCM accept an argument equal to 2^53, as Excel does.

## Context
probes/resource-limits (excel-graph, 2026-10-05, T-0012): C1 `GCD(2^53,2)` = 2 and C3 `LCM(2^53,1)` =
9007199254740992; we give `#NUM!` for both. C2/C4 (2^53-1) already pass. Code:
`MAX_LCM_GCD` in `crates/rustsheet-engine/src/functions/math_and_trigonometry/gcd_lcm.rs` rejects
`>= 2^53`. Excel's actual upper bound is unprobed: add cells for `2^53+2` (exactly representable)
and `2^54` to the probe first, then set the bound from the evidence. Keep a bound (directive 5) and
check that the i64 arithmetic cannot overflow (LCM of two large arguments).
docs/excel-semantics/numbers.md "GCD / LCM argument bound".

The evidence is excel-graph (web) only; for GCD/LCM it contradicts Microsoft's documented
`>= 2^53 → #NUM!`. Confirm with the excel-desktop backend first if it is available
(decision 0019), and record a CONFLICTING entry if desktop differs.

## Acceptance criteria
- [ ] probes/resource-limits C1 and C3 disappear from eval/baseline/probes.txt with no new F lines.
- [ ] Probe cells for the real upper bound added and passing; unit test for LCM overflow.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other integer-argument limits.
