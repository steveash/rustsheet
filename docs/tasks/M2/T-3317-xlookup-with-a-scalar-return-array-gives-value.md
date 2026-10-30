---
id: T-3317
title: "XLOOKUP with a scalar return_array gives #VALUE! where Excel gives if_not_found (t3181 Calc!B11)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`=XLOOKUP(Data!A1,Data!A2,"hit","miss")` (a one-cell lookup array, a scalar text return array)
gives "miss" as in Excel, not #VALUE!.

## Context
Found by T-3210: probes/t3181-collation-prefix-arrays Calc!B11 expects "miss"; ours gives #VALUE!.
The cell is not the prefix rule (the texts differ either way); it is an XLOOKUP argument-shape gap
in lookup_and_reference/xlookup. Check first what a scalar return_array does with a hit.

## Acceptance criteria
- [ ] probes/t3181-collation-prefix-arrays Calc!B11 passes; `make eval-baseline` run, 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
