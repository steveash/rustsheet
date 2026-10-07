---
id: T-0270
title: "VLOOKUP/HLOOKUP lift over an array lookup_value (unblocks T-0222)"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
VLOOKUP and HLOOKUP given an array or multi-cell `lookup_value` look up each element and return a
same-shaped array (spilled), as Microsoft 365 Excel does. Each element's result follows the scalar
rules, including the index checks (T-0211 order: search first, then validate the index).

## Context
This blocks T-0222. Once ROUND lifts over arrays (T-0222), `gen-m2/caa981b0753f455c` Calc!A26
`=HLOOKUP(ROUND(INDEX(Data!A2:B3,1,0),…),Data!B9:D9,Data!C4)` with row index 0 gets an array
lookup_value. Excel gives `#VALUE!`. Today we give `#VALUE!` only by accident: ROUND fails on the
array and HLOOKUP passes the error through. With T-0222 alone we give `#N/A`, because HLOOKUP
searches for the whole array, finds nothing and never reaches the index check. Port attempt of #129,
2026-10-06.

The Excel behavior is not yet evidenced here. Probe it first (excel-probe skill):
- VLOOKUP and HLOOKUP with an array constant, a multi-cell range and an `INDEX(…,1,0)` row as
  lookup_value.
- Exact and approximate match.
- Found, not-found and error elements.
- An out-of-range or 0 index (does every element give `#VALUE!`/`#REF!`?).
- A whole-column lookup_value (allocation bound).

Code: `functions/lookup_and_reference/` (VLOOKUP/HLOOKUP). Reuse the existing element-wise lift
helpers (`functions/text/text_arg.rs`, the IS* `check_range_size` bound in
`math_and_trigonometry/array_size.rs`); do not add a new lifting primitive (directive 3).

## Acceptance criteria
- [ ] Oracle-backed probe case (excel-graph sidecar) covering the cases above; semantics entry in
      `docs/excel-semantics/lookup.md`.
- [ ] VLOOKUP/HLOOKUP lift per the probe. A lifted range above the shared size bound gives the
      DEVIATION error (decision 0014).
- [ ] No new gen-m2 / probes known failures. With T-0222's ROUND lift applied, `caa981b0753f455c`
      Calc!A26 still passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XLOOKUP, MATCH and LOOKUP over array lookup values (file follow-ups if they differ).
- T-0222 itself.
