---
id: T-1933
title: "Add a row-insert op to sheet_ops and turn T-1853's legacy row-insert result into an oracle-filled probe"
milestone: M2
lane: harness
status: todo
priority: P3
size: M
tier: sonnet
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
- [ ] `sheet_ops` has a row-insert op, replayed by the Graph backend and by the eval engine adapter.
- [ ] A probe case with the T-1853 cells is oracle-filled and is cited on references.md "Legacy N and T over a reference-returning call", replacing the one-off note as the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
