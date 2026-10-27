---
id: T-2889
title: "Decision record: how M8's exit criteria are measured (feature oracle suites, the POI corpus subset, M8 probes)"
milestone: M8
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A decision record (`make new-decision`) fixes how each of M8's exit criteria is read from `make eval`'s report at
visible parity (decision 0114), as decision 0085 did for M5, so an M8 exit row and `make exit-owners MILESTONE=M8` can
be built from it. The record names the cells and suites each criterion counts; it adds no gate and changes no baseline.

## Context
- M8's exit text (docs/milestones.md "M8"): "oracle suites for each feature at 100%; POI corpus subset ≥ 99.5%; its
  `probes` cases (decision 0051) at 100% bit-exact". None of the three says which suites, which POI workbooks or cells.
  `MILESTONES` in crates/rustsheet-eval/src/milestone_exit.rs stops at M6 ("M7 to M11 need their own decision first").
- M8's scope lists five features: defined names (workbook, sheet, LAMBDA names), tables and structured references, 3-D
  references, external workbook references (cached values), and sheet add/rename/move with reference rewriting.
- Planner run of 2026-10-27 (`make eval` on 6087413bc; poi, calipers and corpus-bench not fetched):
  - gen-m8a (names and 3-D, T-1606): 12 cases, 3,577 cells, 100%. gen-m8b (tables, T-1607): 9 cases, 3,368 cells,
    100%. Both are `gate = false` with no eval/baseline file (the gating task filed beside this one fixes that).
  - No generated suite covers external references or sheet operations; their evidence is in `probes` (for example
    probes/sheet-ops/*, the external-link cases) and in poi.
  - "Probe milestones (decision 0051)" M8 row: 126 cases, 1,203 scored, 28 known failures, 26 visibly off.
  - `scripts/exit_owners.py` `exit_cells` reads only the suite named `gen-<milestone>`, so `gen-m8a` and `gen-m8b`
    would be missed: the record must say how the row and exit-owners find them.
- Questions the record answers:
  1. Which suites are "oracle suites for each feature", per feature, and whether a feature with no generated suite
     (external references, sheet operations) is measured by its probe cases or needs a suite first (file the task).
  2. Is a minimum cell count required per feature suite (M5 used 20,000, M2 10,000)? "100%" read as 0 visibly off.
  3. Which poi workbooks and cells form the "POI corpus subset": for example cells whose formulas use a defined name,
    a structured reference, a 3-D or external reference (the report's attribution cannot see these today; say what the
    report must add), and how the 99.5% counts deviations and visibly-off cells.
  4. Whether the M8 row needs ironcalc-calc or catalog coverage (M8 has no functions; expected `n/a`).

## Acceptance criteria
- [ ] Decision record added answering questions 1–4 with the 2026-10-27 (or newer) report numbers; docs/milestones.md's
      M8 exit text cites it (edit only the M8 section).
- [ ] A follow-up harness task is filed with `make new-task` for the M8 exit row and exit-owners support, naming the
      record, with concrete thresholds; any missing feature suite or report measure the record requires is filed too.
- [ ] `python3 scripts/tasks.py validate` passes; `make check` green; reviewers approve.

## Out of scope
- Implementing the exit row or the POI-subset measure (the follow-up tasks).
- Gating gen-m8a and gen-m8b (separate task).
- Fixing any M8 cell.
