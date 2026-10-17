---
id: T-1719
title: "An unknown function call keeps the spelling of its name in the formula text"
milestone: M2
lane: engine
status: done
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
- [x] probes/lotus-dot-range-func-second passes and the new probe cells pass; the rule is in docs/excel-semantics/.
- [x] A reload (`Model::from_bytes(model.to_bytes())`) keeps the formula text.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Probed (probes/unknown-function-name-spelling, oracle filled, rule in docs/excel-semantics/references.md
"Unknown function call names"). The task premise ("keeps the spelling as typed") is wrong: Excel
interns the spelling of an unknown call name workbook-wide, first occurrence wins, case-insensitively
(`=foo(1)` then `=Foo(1)` prints `=foo(1)`); cell-ref-shaped `Ab1` prints `AB1`. So the fix needs a
workbook-level spelling registry (like defined-name case), persisted so a reload keeps it; the R1C1
storage currently lowercases (`stringify.rs` LambdaCallKind). That is a design question beyond a
sonnet task. Next: decide where the registry lives (Model, filled in cell order on load/set), update
the stringify to look it up, make probes/lotus-dot-range-func-second and the new probe pass, and
check whether first = load order or edit order (probe by editing).

## Progress (2026-10-16, opus, in flight)
Design read of the code (no code changed yet):
- `=foo(1)` parses as `LambdaCallKind { lambda: NamedVariableKind }` (operands.rs falls back to
  `NamedVariableKind`, lambda.rs `parse_call` wraps it), and stringify.rs prints that callee with
  `name.to_lowercase()` in every mode, A1 and R1C1 storage alike. That lowercasing is the whole
  current failure (probes/unknown-function-name-spelling B4:B18, lotus-dot-range-func-second E2).
- The parser assigns no binding ids (`NamedVariableKind.id` is never `Some`); bound vs free is only
  known by walking LET/LAMBDA scopes, as `prefix_bound_variables` (stringify/bound_variables.rs) does
  for `_xlpm.`. A LET/LAMBDA-bound callee (`LET(f,LAMBDA(x,x),F(1))`) must not take a workbook
  spelling: reuse that walker's scoping (extend it to report free callee names) rather than a copy.
- Proposed shape: a `Model`-level map lowercase name → first spelling, filled when a cell formula is
  parsed into the model (load in cell order, then each edit), and applied to the free callee names of
  the parsed node, so the node itself carries the canonical spelling. Then print the callee as stored
  (drop `to_lowercase`) in both A1 and R1C1: the storage keeps the spelling and a reload rebuilds the
  map in the same order. Check first why the lowercase was there ("distinguish from real function
  calls in R1C1"): a stored `FOO(1)` must still read back as an unknown call, not `NamedFunctionKind`.
- Separate quirk in the same probe: `=Ab1(1)` (a cell-shaped callee) is `#REF!` in Excel and prints
  `=AB1(1)`; ours `#NAME?` (A8). Likely a call of the reference AB1; file it apart if it does not fall
  out of the fix.
- Unprobed: load order vs edit order for "first" (the handoff's last point).

(Resumed again 2026-10-17 by an opus run after 2h idle.)

## Outcome
- `CallNameSpellings` (expressions/parser/call_names.rs), a `Model` field: lowercase name → first
  spelling. Every stored formula's free callee (one no enclosing LET/LAMBDA binds) gets it, on load
  (`parse_formulas`, sheet then shared-formula order, rewriting a stored text it re-spells) and on
  each edit (`shared_formula_index`). Bound callees keep printing lowercase, as before.
- The printer prints a callee as stored. In the R1C1 storage form, a callee that English would
  read back as a function, boolean or cell gets the `_xlpm.` prefix the parser drops (this was what
  the old lowercasing half-did: a Spanish `=TRUE()`/`=SUM(1)` must stay an unknown call, also after
  a reload; that reload case was broken before).
- `walk_names` (stringify/bound_variables.rs) is the one LET/LAMBDA scope walker; `_xlpm.` export
  prefixing and the spelling registry both use it.
- Harness: a spec case's cells are set row by row (an xlsx's load order) so "first met" matches.
- Eval delta: probes known failures 2962 → 2950 (12 fixed: unknown-function-name-spelling B4-B7,
  B9, B10, B14-B18, lotus-dot-range-func-second E2), 0 regressions.
- Follow-up: T-1758 (`=Ab1(1)` is a call of the cell AB1: `#REF!`, printed `=AB1(1)`; A8/B8).

