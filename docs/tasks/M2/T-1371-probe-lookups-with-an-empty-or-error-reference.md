---
id: T-1371
title: "Probe lookups with an empty or error reference-valued lookup_value; give the XMATCH text-vs-number order its own entry"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence that a reference-valued lookup_value (CHOOSE, OFFSET, INDIRECT, A1:A1) pointing at an empty cell or
an error cell behaves like the direct reference in VLOOKUP, HLOOKUP, MATCH, XMATCH, XLOOKUP and LOOKUP; and a separate
lookup.md entry for XMATCH's text-vs-number order.

## Context
T-1256 (PR #556) added `Model::lookup_value_arg` (functions/lookup_and_reference/mod.rs). It dereferences a 1x1
range result, so an empty cell becomes `CalcResult::EmptyCell` and an error cell gives its error. probes/lookup-reference-lookup-value
never points a shape at a blank or an error cell, so no oracle data shows either path (excel-fidelity review MINOR).
The same probe found that `XMATCH("x",{10..50},-1)` is 5 and XLOOKUP gives the last row, while MATCH(..,1) and LOOKUP
give `#N/A`. That finding sits inside the reference-lookup_value entry in docs/excel-semantics/lookup.md, where a
search for XMATCH ordering would miss it.

## Acceptance criteria
- [x] A probe case (excel-graph) adds a blank-cell and a `#DIV/0!`-cell row for each reference shape and lookup function;
  any engine mismatch gets its own task.
- [x] lookup.md has its own heading for the XMATCH/XLOOKUP text-vs-number order, citing the T-1256 probe cells.
- [x] `make check` green; reviewers approve.

## Out of scope
- Multi-cell lookup_value (T-0270).

## Outcome
Added probes/lookup-reference-lookup-value-blank-error (165 formula cells, excel-graph). Excel: an error cell's error
comes back from all eleven forms; a blank key against an all-number table gives `#N/A` (the existing blank-key rules) except XMATCH/XLOOKUP next-smaller mode, which take
the last row. The engine already matched every cell, so no engine task. lookup.md gained the blank/error entry and a
separate XMATCH/XLOOKUP text-vs-number entry. Eval delta: +167 checked, passing cells (165 formulas + 2 error source cells; probes), 0 failures.
