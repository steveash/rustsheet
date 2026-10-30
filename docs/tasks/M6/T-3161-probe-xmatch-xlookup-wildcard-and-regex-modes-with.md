---
id: T-3161
title: "Probe XMATCH/XLOOKUP wildcard and regex modes with a near-equal numeric key"
milestone: M6
lane: functions-lookup
status: in-progress
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
- [ ] A probe case with oracle values for the wildcard and regex modes with a near-equal numeric key; the engine matches or the finding is recorded in lookup.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
