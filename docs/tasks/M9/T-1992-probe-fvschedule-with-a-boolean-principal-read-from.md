---
id: T-1992
title: "Probe FVSCHEDULE with a boolean principal read from a cell reference (T-1765 inferred #VALUE!)"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's result for FVSCHEDULE with a TRUE principal held in a cell is established by a probe.

## Context
T-1765 made a typed TRUE principal `#VALUE!` (probes/fin-fvschedule A18). `fn_fvschedule` reads the principal with `get_number_no_bools`, so a boolean from a reference, INDIRECT or INDEX is `#VALUE!` too, inferred from the ToolPak rule in docs/excel-semantics/financial.md and not probed.

## Acceptance criteria
- [x] a probe case covers `=FVSCHEDULE(C1,{0.1})` with C1=TRUE (and INDEX/INDIRECT); oracle values recorded; implementation changed if Excel differs.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `fin-fvschedule-bool-ref` (excel-graph): a boolean principal from a cell, INDEX or INDIRECT is `#VALUE!`, as implemented; a text-number cell converts. No code change; recorded in financial.md. Eval delta: none (probe cases only).
