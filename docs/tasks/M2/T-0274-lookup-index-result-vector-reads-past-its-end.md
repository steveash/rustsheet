---
id: T-0274
title: "LOOKUP: INDEX result_vector reads past its end like a reference"
milestone: M2
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`LOOKUP(99,A1:A2,INDEX(F1:F1,1))` and `INDEX(F1:G1,1,1)` as result_vector read past the end of the one-cell reference like a plain `F1` (value G1), as in Excel.

## Context
Found by T-0269: probes/lookup-vector-edges Calc!A9 and A12 expect 8 (G1), the engine gives #N/A because INDEX evaluates to a scalar, not a reference, in `lookup_result` (crates/rustsheet-engine/src/functions/lookup_and_reference/lookup.rs). `OFFSET` already works. See lookup.md "LOOKUP vector form". Check how other functions get INDEX's reference result before adding a new path.

## Acceptance criteria
- [ ] probes/lookup-vector-edges passes in full (record with `make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`lookup_result` evaluates an `INDEX(...)` result_vector through `index_impl(.., true)` so it stays a reference and reads past its end like `F1`. probes/lookup-vector-edges now passes in full (probes known failures 260 -> 258, no regressions).
