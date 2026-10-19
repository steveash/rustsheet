---
id: T-1990
title: "Probe a typed or stored one-argument IF(x) and match it"
milestone: M7
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1947]
---

## Goal
What Excel does with a typed or file-stored `=IF(x)` (IF with only a condition) is known and matched.

## Context
T-1947 reviews (MINOR). `Model::if_pick` (functions/logical/mod.rs) gives IF with only a condition what an eta-reduced
IF called with one argument gives (`#VALUE!` per element that is not an error; probes/eta-if-one-argument), and a typed
`=IF(TRUE)` takes the same path; before T-1947 it was an argument-count `#ERROR!`. Whether Excel accepts it typed, or
loads it from a file and with what value, is unprobed (docs/excel-semantics/arrays.md "Functions passed by name").
Probe with the `excel-probe` skill: `=IF(TRUE)`, `=IF(1)`, `=IF({1,0})` set through the oracle (excel-graph may refuse to
enter them; record what it reports), and if refused, note it as entry-only. Then match or record.

## Acceptance criteria
- [ ] A probe case with oracle values (or the recorded refusal) for the typed forms; arrays.md updated.
- [ ] If Excel differs from ours, the typed form is matched without changing the eta path (probes/eta-if-one-argument stays 40/40).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The eta path itself (T-1947).

## Outcome
Probe case `probes/typed-if-one-argument` added. excel-graph refuses every workbook that stores a one-argument `IF(x)`
(501 `FileCorruptTryRepair`), so there are no oracle values and the case stays pending; whether Excel accepts the typed form is
entry-only (desktop question, M14 Phase 4). Recorded in docs/excel-semantics/arrays.md "Functions passed by name". No code
change; eta-if-one-argument unchanged. Eval delta: none.
