---
id: T-1719
title: "An unknown function call keeps the spelling of its name in the formula text"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FORMULATEXT of a call to a name that is no function prints the name as typed, as Excel does: `=SUM(A1.INDEX(B2:B2,1))` stays `=SUM(A1.INDEX(B2:B2,1))` (ours prints `=SUM(a1.index(B2:B2,1))`).

## Context
Found by T-1659: probes/lotus-dot-range-func-second E2 (excel-graph) fails on the case only. `LambdaCallKind` with a `NamedVariableKind` callee prints `name.to_lowercase()` (parser/stringify.rs "When the callee is a plain named variable"), on purpose for the R1C1 storage form. Probe first: a lowercase or mixed-case unknown name (`=foo(1)`, `=Foo(1)`) and whether Excel keeps the case or uppercases it; then make the A1 text keep the spelling while the R1C1 storage still round-trips (a reload must keep it a call of an unknown name, not a LAMBDA call of a variable).

## Acceptance criteria
- [ ] probes/lotus-dot-range-func-second passes and the new probe cells pass; the rule is in docs/excel-semantics/.
- [ ] A reload (`Model::from_bytes(model.to_bytes())`) keeps the formula text.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
