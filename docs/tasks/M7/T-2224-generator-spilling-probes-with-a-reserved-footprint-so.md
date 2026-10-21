---
id: T-2224
title: "Generator: spilling probes with a reserved footprint, so gen suites score spill shapes (synthetic-generator 5.2 shape 3)"
milestone: M7
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-gen` can emit lifted, spilling probes. Each top-level formula whose result is an array
gets an empty footprint to spill into, so a generated suite scores Excel's spill shape and values
(and `#SPILL!` only where a collision is generated on purpose). M7's exit criterion asks for this:
"generated dynamic-array suite (≥ 10k cells, spill shapes included)".

## Context
- Gap triage 2026-10-21 (`main` fce6020d). gen-m7 has 50 cases and 994 scored cells, `gate = false`.
  No open task grows it or adds spill shapes. M6's equivalent is T-2180, M5's was T-2114.
- `generate_case` (tools/src/rustsheet_tools/gen/generator.py) writes formula i to `Calc!A{i}` and checks
  only those cells. A top-level FILTER, SORT, SEQUENCE, VSTACK, TEXTSPLIT, MAP or similar call whose
  result has more than one cell runs into the formula below it. Both Excel and the engine then score
  one `#SPILL!`, so the spill extent and the spilled values are never compared. Wrappers such as
  `COUNT(VSTACK(..))` hide the shape too.
- docs/architecture/synthetic-generator.md §5.2 shape 3 ("Lifted probe") describes the design: the
  probe region reserves a rows x cols empty footprint, and collisions are generated on purpose under
  edge class `spill_blocked`. It is not implemented.
- The harness already scores spill extents for spec cases. With an Excel sidecar, any cell the engine
  stores that the oracle did not record fails against empty (T-1660, `SpillExtent` in
  crates/rustsheet-eval/src/runner.rs). Spilled cells that the oracle records are scored when they are
  in `check` (the probes do this, e.g. probes/m7-argument-modes).
- Catalog: eval/catalog/functions.yaml has no key yet for "this function can return a spill". The
  `dynamic_array: false` field in synthetic-generator.md's schema sketch is not in the catalog, and FILTER
  has only `returns: any`. This task adds it.

## Acceptance criteria
- [ ] The catalog marks every function that can return a multi-cell array: `dynamic_array: true`, as in the
      synthetic-generator.md schema. That covers the M7 functions plus MMULT, MINVERSE, MUNIT, TRANSPOSE,
      FREQUENCY, LINEST and the other array-returning functions. The catalog's tools tests check the key.
- [ ] With a new `--spill` option (or on by default for milestones whose catalog has `dynamic_array`
      functions; the choice is documented in synthetic-generator.md), a generated case lays out each formula
      whose outer call is `dynamic_array: true` in its own block. The block is an anchor cell plus a reserved
      empty footprint (a bounded rows x cols, sized from the data sheet's dimensions, at most 12 x 6 by
      default). Footprints never overlap other formulas or footprints.
- [ ] The case's `check` lists the anchor and every footprint cell, so the oracle sidecar records Excel's
      spilled values and the cells it leaves empty. A footprint cell Excel leaves empty is scored as empty.
- [ ] About 5% of spilling probes get a blocker cell in the footprint (`spill_blocked`), so `#SPILL!` stays
      covered. The fraction is set in the generator and documented.
- [ ] Generation stays deterministic per seed (same seed, same bytes) and tools tests cover the layout:
      no overlaps, `check` covers each footprint, blocker placement.
- [ ] synthetic-generator.md §5.2 says shape 3 is implemented and names the option.
- [ ] Existing generated suites are unchanged (no regeneration in this task).
- [ ] `make check` green (ruff, mypy strict, tools tests); reviewers approve.

## Out of scope
- Generating or filling gen-m7 (T-2225).
- Footprints larger than the data sheet (whole-column spills): bounded by design, see arrays.md for the
  directive-5 bound.
