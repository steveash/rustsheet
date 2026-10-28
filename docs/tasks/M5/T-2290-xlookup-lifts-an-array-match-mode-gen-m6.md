---
id: T-2290
title: "XLOOKUP lifts an array match_mode (gen-m6 fe0972380d030e4a A224 is #SPILL!)"
milestone: M5
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XLOOKUP given an array `match_mode` returns an array, as Excel does, so gen-m6/fe0972380d030e4a Calc!A224 is `#SPILL!`.

## Context
- Left over from T-2159 (gen-m6 triage of T-2180): `XLOOKUP(VALUE(..),Data!A5:A10,Data!C1:D9,ISERR(..),DECIMAL(OFFSET(Data!B10:E11,5,3,3,2),21),0)`.
  DECIMAL now lifts and returns a 3x2 array (T-2159). Excel gives `#SPILL!`, so XLOOKUP lifts its match_mode over that
  array. Ours is `#VALUE!` (`make eval-show CASE=gen-m6/fe0972380d030e4a.case.json`, 1 line in eval/baseline/gen-m6.txt).
- Before coding, check docs/excel-semantics/lookup.md "Lookup and reference function argument modes" for XLOOKUP's
  match_mode and search_mode. If they are unprobed, probe them with the `excel-probe` skill: an array constant, and a
  range given directly, for match_mode and for search_mode. Then put the arguments on the SPEC-0011 driver or lift them
  in XLOOKUP's own reader.

## Acceptance criteria
- [ ] gen-m6/fe0972380d030e4a Calc!A224 leaves eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure.
- [ ] lookup.md records XLOOKUP's match_mode and search_mode argument kinds, with their evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The lookup_value, lookup_array and return_array readers.

## Outcome
XLOOKUP no longer returns a scalar error lookup_value early when match_mode or search_mode is an array; the error lifts per element, so the blocked spill is `#SPILL!`. gen-m6/fe0972380d030e4a Calc!A224 now passes (baseline shrank by 1). lookup.md records the argument kinds.
