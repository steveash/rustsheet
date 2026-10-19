---
id: T-1933
title: "Add a row-insert op to sheet_ops and turn T-1853's legacy row-insert result into an oracle-filled probe"
milestone: M2
lane: harness
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
T-1853's row-insert result (a legacy `=N(IF(1,{2;3}))+A5` stays a plain `<f>` and stays `#VALUE!` after a row is inserted above it) is an oracle-filled probe case that scores the engine's SPEC-0012 rule 6 implementation.

## Context
T-1853 confirmed the row insert in a one-off excel-graph session (docs/research/t1853-legacy-cell-edits-2026-10-18.md). Graph
`worksheets('S')/range(address='1:1')/insert` with `{"shift": "Down"}` worked in a persistent session. The result is not kept as a case.
rustsheet-case/v1 already replays sheet edits through `sheet_ops`:
- spec2xlsx records them (tools/src/rustsheet_tools/spec2xlsx.py);
- `GraphBackend._replay_sheet_ops` replays them before the FullRebuild (oracle/backends.py);
- `apply_sheet_ops` mirrors them in the engine (crates/rustsheet-eval/src/engine.rs);
- eval/suites/probes/cases/sheet-ops/ holds the existing cases.
Add a row-insert op on all three sides. Then add a probe case (tag M2) with the B2/C2 cells of the research note marked under
`legacy_formulas`, and fill it with `rustsheet-oracle fill -b excel-graph`.

## Acceptance criteria
- [x] `sheet_ops` has a row-insert op, replayed by the Graph backend and by the eval engine adapter.
- [x] A probe case with the T-1853 cells is oracle-filled and is cited on references.md "Legacy N and T over a reference-returning call", replacing the one-off note as the evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Sonnet run stopped before coding: the engine side needs a design call.
- Python side is mechanical: add `"insert_row"` (JSON `{"op":"insert_row","sheet":"S","row":1}`; carry the 1-based row in the `to` slot of the `(op, sheet, to)` tuple) to `SHEET_OPS`/`sheet_op()` in spec2xlsx.py, replay it in `GraphBackend._replay_sheet_ops` with `POST worksheets('S')/range(address='<n>:<n>')/insert` `{"shift":"Down"}` (research note method), and extend tests/test_spec2xlsx_sheet_ops.py, `SheetOp` in case.rs, and docs/architecture/eval-harness.md.
- Blocker: `apply_sheet_ops` (engine.rs) runs through the `rustsheet` facade, and `Txn` has no row insert (SPEC-0012 line 257: "None of these is reachable from the rustsheet facade today"). `Workbook.model` is `pub(crate)`. Options: (a) add `Txn::insert_rows` to the facade, which needs undo for a structural edit and touches SPEC-0001 (api lane); (b) a narrow harness-only hook. Pick one (probably a decision record), then do the rest.
- `rustsheet-oracle status` showed excel-graph OK, so the fill (`rustsheet-oracle fill -b excel-graph`) should work.

## Outcome
- What changed:
  - New `{"op":"insert_row","sheet":"S","row":n}` sheet op (one blank row above the 1-based row).
  - spec2xlsx records it and the Graph backend replays it (`range(address='n:n')/insert`, shift Down).
  - The eval adapter replays it through the harness-only `Workbook::insert_rows` (cargo feature `harness_row_insert`, decision 0076), then recalculates.
  - The spill-extent check now knows inputs at their moved addresses.
- New probe, probes/sheet-ops/sheet-ops-insert-row-legacy-cell (excel-graph, 2026-10-19):
  - It confirms T-1853: after the insert the legacy B3 is still `#VALUE!`, C3 is 2, FORMULATEXT shows `=N(IF(1,{2;3}))+A6`, and the dynamic twin spills 3, 4.
  - It replaces the one-off note as the evidence in references.md.
- Eval delta: probes +1 case, 2 new-coverage known failures (B3, B4). Ours loses the legacy marker on a row insert and spills; T-1852 implements the fix. 0 regressions in every suite.
