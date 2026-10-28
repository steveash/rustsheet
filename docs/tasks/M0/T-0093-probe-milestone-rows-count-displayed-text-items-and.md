---
id: T-0093
title: "Probe-milestone rows count displayed-text items and list the failing ones as case!Sheet!A1"
milestone: M0
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-0088]
---

## Goal
Each row of the report's "Probe milestones" table (decision 0051) counts the displayed-text items of its cases
(`CaseResult::text`, T-1610) and lists the failing ones, so M10's exit row can require 0 failing text items in the
M10 probes (decision 0146 item 4(b)).

## Context
- Decision 0146 (T-0088, PR #2144) item 4: "the report adds text-item counts and failing text cells to each probe
  row"; exit-owners names failing text items as `case!Sheet!A1`, like value cells. This task is that report input
  only; the M10 exit row is T-0097.
- `crates/rustsheet-eval/src/probe_milestones.rs` `MilestoneRow` counts value cells only (`cells`, `bit_exact`,
  `known_failures`, `visibly_off`, `visibly_off_cells`). Text items live in `runner::CaseResult::text`
  (`text_expect::TextResult { sheet, cell, pass, .. }`); suites already total them as `text_cells` /
  `text_passed` in report.rs, and report.md's "Displayed text" section lists failing items. Reuse those, and
  the `case!Sheet!A1` naming `visibly_off_cells` uses (directive 3).
- `make eval` at abefb502: probes has 410 text items, all passing, so every row's new failing count is 0 today.
- Decision 0051 assigns a case one milestone; a text item takes its case's milestone (a `probes/catalog/*` cell's
  text item takes that cell's milestone, as its value cell does).

## Acceptance criteria
- [ ] `MilestoneRow` gains `text_items: usize`, `text_failing: usize` and `text_failing_cells: Vec<String>`
      (`#[serde(default)]`), filled from the cases the row already assigns.
- [ ] Unit tests: a case with two text items, one failing, adds 2 / 1 and the failing item's `case!Sheet!A1` to
      its milestone's row and nothing to another row; a case with no text items leaves the counts at 0.
- [ ] report.md's "Probe milestones" table gains `text items` and `failing text` columns; `make eval` on `main`
      shows 410 text items in total across rows and 0 failing.
- [ ] No change to baselines, gates or `scripts/`; `make check` green; reviewers approve.

## Out of scope
- The M10 exit row and exit-owners support (T-0097).
- Scoring text items in the eval gate (they stay information, T-1610).
