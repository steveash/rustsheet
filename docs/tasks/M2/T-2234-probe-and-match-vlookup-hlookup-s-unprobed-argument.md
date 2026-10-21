---
id: T-2234
title: "Probe and match VLOOKUP/HLOOKUP's unprobed argument edges: two lifted errors, blank index elements, a fractional index at the -2^31 cut-off, a range index's spill"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP and HLOOKUP give Excel's value or error, backed by oracle evidence, on the argument edges
that docs/excel-semantics/lookup.md and the code still call unprobed. Today the engine guesses
them.

## Context
Gap triage 2026-10-21. VLOOKUP and HLOOKUP are M2 functions with a high prior. All their scored
cells pass, but four behaviors rest on guesses that no case checks:
1. **Two errors in one lifted pair.** lookup.md "VLOOKUP / HLOOKUP with an array index or
   range_lookup" says which error wins when a lookup_value element and a range_lookup element are
   both errors is unprobed. The engine gives the lookup_value's error. Example:
   `VLOOKUP({#N/A;2},A1:B4,2,{#DIV/0!;TRUE})`, and the scalar `VLOOKUP(NA(),A1:B4,2,1/0)` next to it.
   `lookup_head` in `functions/lookup_and_reference/index_arg.rs` has the same note for the
   unlifted call ("Which error wins when several arguments fail is unprobed"). lookup.md
   "VLOOKUP / HLOOKUP search before validating the index" settles lookup_value before the index
   before range_lookup only for some pairs. Add a table_array error (`VLOOKUP(NA(),1/0,..)`, a
   `#REF!` table) to the same grid.
2. **Blank elements of a computed index array.** Same lookup.md entry: "blank elements of a
   computed index array (read as 0, `#VALUE!`)" are unprobed. Example: `VLOOKUP(2,A1:B4,IF({1,0},2,Z1))`
   with Z1 blank, and `INDEX` of a blank-holding range used as the index.
3. **A fractional index next to the 32-bit cut-off.** lookup.md: -2147483648 is `#REF!` and
   -2147483647 is `#VALUE!` (probes/lookup-index-shapes H19-H36), but a fractional index next to
   the cut-off is unprobed. `index_arg.rs` compares `number <= f64::from(i32::MIN)` before
   truncation. Probe -2147483647.5, -2147483648.5, -2147483647.9999999 and the same on the
   positive side past the table, found and not found, for VLOOKUP and HLOOKUP.
4. **A multi-cell range index outside the intersection rows.** lookup.md: `VLOOKUP(2,A1:C3,E1:E2)`
   unspilled in J10 is `#REF!`, and "whether it spills into J11 is unprobed". Probe it with J11
   empty and with J11 filled, and from a row inside E1:E2 (probes/lookup-index-multicell-lifted
   holds the lifted variants).

Write one excel-probe case (excel-graph oracle) with about 40 cells, giving each formula its own row
so that no spill collides by accident. Record the rules in lookup.md (each "Unprobed" sentence above
becomes a rule with its cells). Change `lookup_each.rs` / `index_arg.rs` only where Excel differs.

## Acceptance criteria
- [ ] A new probes case, oracle-filled (excel-graph), covers the four groups for VLOOKUP and
      HLOOKUP, found and not found. Every cell passes, or its known failure is recorded with `make
      eval-baseline` and has a filed task named in the Outcome.
- [ ] lookup.md has no "unprobed" sentence left for these four points; each cites its cells.
- [ ] Any engine change has unit tests; no gated baseline gains a line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XLOOKUP, MATCH and LOOKUP error order (their own lookup.md entries).
- An array of references as the table (SPEC-0014, T-2064).
