---
id: T-1719
title: "An unknown function call keeps the spelling of its name in the formula text"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
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

## Handoff
Probed (probes/unknown-function-name-spelling, oracle filled, rule in docs/excel-semantics/references.md
"Unknown function call names"). The task premise ("keeps the spelling as typed") is wrong: Excel
interns the spelling of an unknown call name workbook-wide, first occurrence wins, case-insensitively
(`=foo(1)` then `=Foo(1)` prints `=foo(1)`); cell-ref-shaped `Ab1` prints `AB1`. So the fix needs a
workbook-level spelling registry (like defined-name case), persisted so a reload keeps it; the R1C1
storage currently lowercases (`stringify.rs` LambdaCallKind). That is a design question beyond a
sonnet task. Next: decide where the registry lives (Model, filled in cell order on load/set), update
the stringify to look it up, make probes/lotus-dot-range-func-second and the new probe pass, and
check whether first = load order or edit order (probe by editing).
