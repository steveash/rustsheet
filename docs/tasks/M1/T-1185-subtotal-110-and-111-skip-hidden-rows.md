---
id: T-1185
title: "SUBTOTAL 110 and 111 skip hidden rows"
milestone: M1
lane: functions-math
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-1036]
---

## Goal
SUBTOTAL 110 (VAR.S) and 111 (VAR.P) skip hidden and filtered rows like the other 101-111 function numbers.

## Context
probes/subtotal-hidden-filtered (excel-graph, T-1036): Hide!D10 `=SUBTOTAL(110,A1:A8)` is 2444.7 and Hide!D11 `=SUBTOTAL(111,A1:A8)` is 2037.25 with rows 2 and 5 hidden; the engine gives the full-range 1959.55 and 1714.61 because `fn_subtotal` (crates/rustsheet-engine/src/functions/subtotal.rs) maps 110 and 111 to `SubTotalMode::Full`. docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows". Also check Hide!C10 (`SUBTOTAL(10,..)`, one ulp off: 1959.5535714285716 vs Excel 1959.5535714285713); if VAR.S summation order explains it, fix it here or file it.

## Acceptance criteria
- [ ] Hide!D10 and Hide!D11 pass; `make eval-baseline` records them.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
