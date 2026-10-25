---
id: T-2644
title: "Probe GROUPBY/PIVOTBY eta functions whose minimum is 0 (_xleta.ROW, COLUMN, SHEET, RANDARRAY, PI)"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence settles what GROUPBY and PIVOTBY return for an eta function whose catalog minimum is 0,
so the engine's whole-result `#VALUE!` for them is either confirmed or corrected.

## Context
T-1706 (#1062) looks up each eta function's catalog minimum argument count (`eta_min_args.rs`,
`eta_arity` in `functions/lookup_and_reference/group_functions.rs`). A function with minimum 0 now
makes the whole result `#VALUE!`. Only NOW was probed (`probes/groupby-eta-arity`).
`docs/excel-semantics/arrays.md` marks ROW, COLUMN, SHEET, SHEETS and RANDARRAY as unprobed and
extrapolated from NOW; PI, TODAY, NA, RAND and TRUE/FALSE are extrapolated too, but arrays.md does not
name them. ROW and COLUMN take an optional reference, so Excel
may pass them the group's values instead. Found by the 2026-10-25 steward spot-audit (architecture
reviewer on PR #1062).

## Acceptance criteria
- [ ] An oracle-filled probe (`excel-probe` skill) covers `_xleta.ROW`, `_xleta.COLUMN`, `_xleta.SHEET`,
  `_xleta.SHEETS`, `_xleta.PI` and `_xleta.RANDARRAY` (wrapped so the value is deterministic, e.g. ROWS/ISERROR) in
  GROUPBY and PIVOTBY.
- [ ] arrays.md records the result; any mismatch gets a functions-lookup fix task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (the fix task does that).
