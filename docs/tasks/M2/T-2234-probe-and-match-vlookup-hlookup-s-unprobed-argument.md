---
id: T-2234
title: "Probe and match VLOOKUP/HLOOKUP's unprobed argument edges: two lifted errors, blank index elements, a fractional index at the -2^31 cut-off, a range index's spill"
milestone: M2
lane: functions-lookup
status: done
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
Gap triage 2026-10-21. VLOOKUP and HLOOKUP are M2 functions with a high prior. No failing cell is
VLOOKUP's or HLOOKUP's own miss (the 6 failing cells that contain them miss in DECIMAL, SUMPRODUCT, ARABIC
or NPV), but four behaviors rest on guesses that no case checks:
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
   the cut-off is unprobed. `index_arg.rs` compares `number <= f64::from(i32::MIN)` and uses
   `number.floor()`, while lookup.md says a fractional element truncates: the two differ on these
   negative fractions (-2147483647.5 floors to -2147483648), so check floor against trunc. Probe -2147483647.5, -2147483648.5, -2147483647.9999999 and the same on the
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

## Outcome
Done 2026-10-21. Two new excel-graph probes: `probes/lookup-argument-edges` (153 cells, the four groups for VLOOKUP and
HLOOKUP, found and not found) and `probes/lookup-index-near-integer` (1,095 cells, indexes just below and above an
integer at 2, 3, 50 and 100). Before the fix, 22 of the 153 cells failed. Now both cases pass in full.
- A table_array error comes after lookup_value and before the index and range_lookup, and a lifted lookup_value spills
  it per element, as does a lifted range_lookup (`lookup_head`, `table_error_each` in index_arg.rs).
- The index is floored with INDEX's near-integer snap (`near_integer_count`, 2^-22 + 2^-33), and the -2^31 cut-off is
  tested on the floored value: -2147483647.5 is `#REF!` and 3.9999999 is column 4.
- The rest was already right in the engine: the two-error pairs (lookup_value wins), blank index elements (`#VALUE!`
  for the whole call) and a range index's spill. lookup.md now records them with cells. No "unprobed" sentence is left
  for the four points.
- Eval delta: the gated probes suite gains 2 cases (1,248 cells), all passing. 22 of them failed before the fix. 0
  regressions, no baseline line changed.
