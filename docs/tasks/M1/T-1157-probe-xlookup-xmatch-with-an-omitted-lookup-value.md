---
id: T-1157
title: "Probe XLOOKUP/XMATCH with an omitted lookup_value: blank or empty text, linear and binary"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles whether an omitted lookup_value in XLOOKUP/XMATCH behaves as a blank cell or as empty text, in linear (exact, match_mode -1/1, wildcard) and binary search.

## Context
docs/excel-semantics/lookup.md ("omitted lookup_value", T-1153) rests on one Excel-authored cell, XLOOKUP.xlsx Errors!N1 `XLOOKUP(,L:L,M:M,,1,2)` = 6. `linear_scan` (functions/lookup_and_reference/xmatch.rs) treats only an `EmptyCell` lookup value as blank, so `EmptyArg` takes the non-blank route in linear modes. Write a probe next to probes/xmatch-blank-extensions with `XMATCH(,D1:D3)`, `XMATCH(,A:A,1)`, `XLOOKUP(,{"";1},{1;2})` and binary variants; fill with the oracle; if the engine differs, file the fix as its own task.

## Acceptance criteria
- [ ] A probe case with oracle values covers exact, -1/1, wildcard and binary modes with an omitted lookup_value, and lookup.md records each finding or the fix task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/xlookup-omitted-lookup-value (40 cells, excel-graph). An omitted lookup_value equals a blank-cell reference in all
exact, -1/1, wildcard, reverse and binary modes; empty text differs. The engine matches in binary mode but not in linear modes
(9 cells), filed as T-1188. Recorded in docs/excel-semantics/lookup.md. Eval delta: +9 known failures (new case, all tracked by T-1188), 0 regressions.
