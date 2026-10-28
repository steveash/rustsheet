---
id: T-2926
title: "One helper for the case!Sheet!A1 cell id in eval reports"
milestone: M2
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every report field that names a cell as `case!Sheet!A1` is built by one helper, so the strings that
`scripts/exit_owners.py` compares across report sections (decision 0135) cannot drift apart.

## Context
- The reviewer-quality review of T-2822 (MINOR) found the same `format!("{}!{}", r.id, c.key())` in
  three places: crates/rustsheet-eval/src/function_scope.rs (the function-scope rows' `failing` and
  `visibly_off`), milestone_exit.rs (`gen_visibly_off_cells`) and probe_milestones.rs
  (`visibly_off_cells`). `exit_owners.py` matches those strings exactly.
- Search for other copies with `rg 'format!\("\{\}!\{\}"'` in crates/rustsheet-eval before adding the helper (for example next to
  `CellResult::key` in runner.rs).

## Acceptance criteria
- [ ] One function builds `case!Sheet!A1` from a case id and a cell; the three call sites above, and any
      other copy found, use it.
- [ ] report.json is byte-for-byte unchanged on a `make eval` run (no field changes).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the cell-id format, or how sheet names are quoted.
