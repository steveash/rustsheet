---
id: T-2746
title: "Probe XLOOKUP error lookup_value with an unblocked array match_mode"
milestone: M5
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's per-element values are known for XLOOKUP with an error lookup_value and an array match_mode or search_mode, and lookup.md records them.

## Context
T-2290 lifts the error per element, evidenced only by a blocked spill (`#SPILL!`). Use the `excel-probe` skill with unblocked spills, e.g. `=XLOOKUP(1/0,A1:A3,B1:B3,,{0;1})` and `=XLOOKUP(NA(),A1:A3,B1:B3,,0,{1,9})`, plus an invalid mode element, to learn each element's error and which wins. Code: `xlookup_impl` / `xlookup_one` (functions/xlookup.rs).

## Acceptance criteria
- [x] Probe cells and oracle values recorded; lookup.md's UNVERIFIED note under "Lookup and reference function argument modes" resolved; our behavior matches.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed 16 formulas (32 cells) on excel-graph: an error lookup_value fills every element and beats an invalid mode element; if_not_found does not replace it. lookup.md updated; ours already matched, so the eval baseline is unchanged. Evidence: probes/t2746-xlookup-error-lookup-array-modes.
