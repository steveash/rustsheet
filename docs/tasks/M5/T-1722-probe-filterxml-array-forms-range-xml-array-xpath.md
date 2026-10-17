---
id: T-1722
title: "Probe FILTERXML array forms: range xml, array xpath, error elements"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FILTERXML's array forms beyond a first-argument array literal are probed against Excel and the engine matches.

## Context
T-1704 lifts FILTERXML over arrays; only `FILTERXML({...;...},"//a")` is oracle-backed. Unprobed: `FILTERXML(A1:A2,"//a")`, `FILTERXML("<a/>",{"//a","//b"})`, `FILTERXML({"x";#N/A},"//a")`, `FILTERXML(NA(),{"//a";"//b"})`. Also `FILTERXML(LAMBDA(x,x),NA())` (a lambda first argument now gives #VALUE! before the second argument is evaluated). See docs/excel-semantics/web.md; code in functions/text/filterxml.rs.

## Acceptance criteria
- [x] Probe cases with oracle values for the four forms above; engine fixed where it differs; web.md updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added probes/filterxml-array-forms with excel-graph oracle values for all five forms; the engine (T-1704) already matched every cell, so no code change. Recorded in docs/excel-semantics/web.md. probes known failures unchanged (2281), new case passes.
