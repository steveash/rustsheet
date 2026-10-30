---
id: T-3161
title: "Probe XMATCH/XLOOKUP wildcard and regex modes with a near-equal numeric key"
milestone: M6
lane: functions-lookup
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Establish with the oracle how XMATCH and XLOOKUP wildcard (2) and regex (3) modes treat a numeric key a few ULP from a table number, and match it.

## Context
T-3132 made the exact, next-smaller and next-larger modes compare exact doubles. The wildcard mode with a numeric key still goes through `compare_values` (15 digits) in `linear_scan` (xmatch.rs); lookup.md lists it as unprobed. Probe e.g. `XMATCH(1,{2;1+2^-52;0.5},2)` and the regex equivalents (excel-probe skill).

## Acceptance criteria
- [x] A probe case with oracle values for the wildcard and regex modes with a near-equal numeric key; the engine matches or the finding is recorded in lookup.md.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- probes/t3161-xmatch-wildcard-regex-near-equal (73 cells, excel-graph): XMATCH/XLOOKUP wildcard mode with a number
  key compares exact doubles, as the exact mode does (`XMATCH(1,{2;1+2^-52;0.5},2)` is #N/A); the regex mode reads
  numbers as text, so near-equal numbers match there.
- `linear_scan` (xmatch.rs) now compares a wildcard-mode number key with `compare_searched` instead of the 15-digit
  `compare_values`. Regex mode already matched.
- Eval delta: +1 case, 73/73 (18 cells failed before the fix); no regressions. lookup.md updated.
