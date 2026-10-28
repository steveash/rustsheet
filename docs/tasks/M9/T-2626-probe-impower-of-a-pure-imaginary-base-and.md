---
id: T-2626
title: "Probe IMPOWER of a pure imaginary base and a lopsided base whose x^2 overflows (T-2539 follow-up)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
IMPOWER of a pure imaginary base (`IMPOWER("1E+300i",-0.5)`) and of a lopsided base (`IMPOWER("1E+200+1E-300i",-0.5)`) match Excel, with the rule oracle-evidenced.

## Context
T-2539 made IMPOWER `#NUM!` when both parts are nonzero and x²+y² overflows (probes/eng-impower-overflow). Pure imaginary and lopsided bases are unprobed; the guard in `fn_impower` (complex/mod.rs) lets them through. Probe both shapes with excel-probe and adjust the guard if Excel differs; record in docs/excel-semantics/engineering.md.

## Acceptance criteria
- [ ] Probe case oracle-filled (excel-graph) and passing; `fn_impower` matches it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
