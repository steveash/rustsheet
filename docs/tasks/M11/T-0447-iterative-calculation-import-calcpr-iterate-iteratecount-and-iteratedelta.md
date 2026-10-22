---
id: T-0447
title: "Iterative calculation: import calcPr iterate, iterateCount and iterateDelta and sweep cycles in place"
milestone: M11
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-0446]
---

## Goal
A workbook saved with iterative calculation on (`<calcPr iterate="1" iterateCount=… iterateDelta=…>`)
calculates its circular references as Excel does: the cycle's cells are swept in row-major sheet order,
in place, up to iterateCount times, stopping at the first sweep whose largest change is below
iterateDelta. They are no longer `#CIRC!`.

## Context
Found by the mog head-to-head (docs/research/mog-head-to-head-2026-10-22.md):
- Two real financial models in the Calipers corpus, pv23_021 and pv26_004 (T-0446), are saved with
  `iterate="1"`. Rustsheet makes about 3,600 of their formula cells `#CIRC!`, its largest
  wrong-type group on that corpus. Mog and Excel compute numbers.
- The importer ignores `iterate`, `iterateCount` and `iterateDelta`, and the engine has no iterative
  mode, so every cycle cell is an error.

The rules are already established, with probes:
- docs/excel-semantics/recalc.md, "Iterative calculation sweeps the cycle in sheet order, in place" and
  "iterateDelta stops the iterations for the whole workbook, not per cycle" (both VERIFIED, excel-graph).
- probes/circular-iteration-on, -limits, -delta, -delta-small, -delta-no-reader, -delta-boundary.
- The harness already runs spec cases with `calculation.iterate` three times, the oracle's count
  (T-0429, `calculations` in crates/rustsheet-eval/src/engine.rs).
- spec2xlsx writes the `calcPr` attributes, so the probes and the Calipers models exercise the same
  import path.

Code: the cycle detection and the `#CIRC!` paths are in crates/rustsheet-engine/src/model.rs
(`run_pass`); calcPr import is in crates/rustsheet-xlsx/src/import.

Coordinate with:
- T-0445 (iteration off: cycle cells and their readers are 0). It changes the same detection path;
  whichever lands second rebases onto the other.
- T-0401 (SPEC-0002). Do not build a dependency graph here. Sweeping the cells found on a cycle during
  a pass is enough for Excel's order.

Directive 5: the sweep count is bounded by iterateCount (Excel caps it at 32767), and each sweep by the
existing per-pass work budget.

## Acceptance criteria
- [ ] The probes/circular-iteration-on, -limits, -delta, -delta-small, -delta-no-reader and
      -delta-boundary lines leave eval/baseline/probes.txt (`make eval-baseline`).
- [ ] calipers/pv_goldens/pv23_021 and pv26_004 have no `#CIRC!` cells. Their known failures in
      eval/baseline/calipers.txt drop (`rustsheet-eval update-baseline --suite calipers`) accordingly, with no new known failure in any suite.
- [ ] A round-trip test keeps `iterate`, `iterateCount` and `iterateDelta` on export.
- [ ] recalc.md's two "Ours" lines state the new behavior.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cycles with iteration off (T-0445).
- Data tables inside a cycle (T-0441).
