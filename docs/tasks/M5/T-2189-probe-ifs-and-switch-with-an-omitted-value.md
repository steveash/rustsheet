---
id: T-2189
title: "Probe IFS and SWITCH with an omitted value argument"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what IFS and SWITCH give when the value they pick is an omitted argument (`IFS(TRUE,)`, `SWITCH(1,1,)`), recorded in docs/excel-semantics/lookup.md.

## Context
T-2168 made IFS and SWITCH turn the empty value into 0 (`CalcResult::empty_value_as_zero`,
crates/rustsheet-engine/src/functions/logical/{mod.rs,switch.rs}), on probes/lookup-blank-result-to-caller O1, O4,
O9, O10. An omitted value argument is the same `EmptyArg`, so the engine gives 0 for `IFS(TRUE,)&""` too, but Excel
for the web refused a workbook holding `IFS(TRUE,)` (createSession 501), so it is unprobed. Try a probe with
`IFS(TRUE,)&""`, `SWITCH(1,1,)&""`, `ISBLANK(IFS(TRUE,))` and `COUNT(SWITCH(1,1,))`, one per case so a refusal names
the formula; if excel-graph refuses them all, record that and put the question in M14 (decision 0039).

## Acceptance criteria
- [x] Oracle values for the cells above (or the refusal recorded per formula) in lookup.md "VLOOKUP and HLOOKUP hand a blank result cell to their caller as 0".
- [x] An engine follow-up task filed for any mismatch.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine.

## Outcome
Four probes (probes/t2189-*, one formula each) were accepted by excel-graph: `IFS(TRUE,)&""` "0", `SWITCH(1,1,)&""` "0",
`ISBLANK(IFS(TRUE,))` FALSE, `COUNT(SWITCH(1,1,))` 1. The engine already matches all four (eval show: Pass), so no engine follow-up.
Recorded in lookup.md; the earlier 501 note is narrowed (it was a larger workbook).
