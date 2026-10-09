---
id: T-1224
title: "Probe called LET names bound to unions, erroring arguments, empty elements and other reference-returning functions"
milestone: M1
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's results are on record for the called-LET-name and LAMBDA-parameter forms that T-1092 documented without evidence.

## Context
Steward spot-audit 2026-10-09 of PR #453 (T-1092), reviewers excel-fidelity, architecture and quality on
opus. docs/excel-semantics/references.md states more than the probes show:
- "#REF! whatever the reference": a union binding gives `#VALUE!` in the engine
  (`=LET(f,(A1,A2),f(2))`, `=LAMBDA(f,f(2))((A1,A2))`; `Binding::Areas` -> `several_areas`), not probed.
- "without the arguments mattering": only `f()` vs `f(2)` was probed; erroring arguments were not
  (`=LET(f,"a",f(1/0))`, `=LET(f,1,f(NA()))`, `=LET(f,{1,2},f(1/0))`).
- Empty elements map to `#NAME?` (`lambda.rs` ~237-240) without evidence; try `=LAMBDA(f,f(2))()`.
- Other reference-returning functions as the bound value: `=LET(f,XLOOKUP(2,A1:A3,B1:B3),f(2))`,
  `TAKE(A1:A3,1)`, `DROP`, `OFFSET(A1,0,0)`, `TRIMRANGE` (VLOOKUP/HLOOKUP bind a value, INDEX a
  reference: probes/let-bind-reference-forms).
Write the probe cases (excel-probe skill), get excel-graph values, record each rule in references.md
(remove claims the oracle contradicts) and file engine tasks for mismatches.

## Acceptance criteria
- [ ] A probe case covering every form above is oracle-filled and committed.
- [ ] references.md states only what the evidence shows; each engine mismatch has a filed task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (file tasks).
